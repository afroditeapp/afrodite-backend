mod limits;
mod notification;
mod report;

use std::sync::Arc;

use database_chat::current::{
    read::GetDbReadCommandsChat,
    write::{
        GetDbWriteCommandsChat,
        chat::{ChatStateChanges, ReceiverBlockedSender},
    },
};
use error_stack::ResultExt;
use model_chat::{
    AccountIdInternal, AddPublicKeyResult, ChatStateRaw, ClientId, ClientLocalId,
    MatchesIteratorSessionIdInternal, NewReceivedLikesCount, PendingMessageId,
    PendingMessageIdInternal, PendingNotificationFlags, PublicKeyId,
    ReceivedLikesIteratorSessionIdInternal, ReceivedLikesSyncVersion, SendMessageResult,
    SentMessageId, SyncVersionUtils,
};
use server_data::{
    DataError, DieselDatabaseError, IntoDataError, app::EventManagerProvider, db_transaction,
    define_cmd_wrapper_write, id::ToAccountIdInternal, read::DbRead, result::Result,
    write::DbTransaction,
};
use simple_backend_utils::ContextExt;
use utils::encrypt::ParsedKeys;

use crate::{cache::CacheWriteChat, read::GetReadChatCommands};

define_cmd_wrapper_write!(WriteCommandsChat);

impl<'a> WriteCommandsChat<'a> {
    pub fn report(self) -> report::WriteCommandsChatReport<'a> {
        report::WriteCommandsChatReport::new(self.0)
    }
    pub fn notification(self) -> notification::WriteCommandsChatNotification<'a> {
        notification::WriteCommandsChatNotification::new(self.0)
    }
    pub fn limits(self) -> limits::WriteCommandsChatLimits<'a> {
        limits::WriteCommandsChatLimits::new(self.0)
    }
}

impl WriteCommandsChat<'_> {
    pub async fn modify_chat_state(
        &self,
        id: AccountIdInternal,
        action: impl Fn(&mut ChatStateRaw) + Send + 'static,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().modify_chat_state(id, action)?;
            Ok(())
        })
    }

    /// Like or match a profile.
    ///
    /// Returns Ok only if the state change happened.
    pub async fn like_or_match_profile(
        &self,
        id_like_sender: AccountIdInternal,
        id_like_receiver: AccountIdInternal,
    ) -> Result<SenderAndReceiverStateChanges, DataError> {
        db_transaction!(self, move |mut cmds| {
            let interaction = cmds
                .chat()
                .interaction()
                .get_or_create_account_interaction(id_like_sender, id_like_receiver)?;

            let updated = if interaction.is_like()
                && interaction.account_id_sender == Some(id_like_sender.into_db_id())
                && interaction.account_id_receiver == Some(id_like_receiver.into_db_id())
            {
                return Err(DieselDatabaseError::AlreadyDone.report());
            } else if interaction.is_like()
                && interaction.account_id_sender == Some(id_like_receiver.into_db_id())
                && interaction.account_id_receiver == Some(id_like_sender.into_db_id())
            {
                let next_id = cmds.chat().upsert_next_match_id()?;
                let account_and_conversation_id = (
                    id_like_sender,
                    cmds.chat().upsert_next_conversation_id(id_like_sender)?,
                );
                let another_conversation_id =
                    cmds.chat().upsert_next_conversation_id(id_like_receiver)?;
                interaction
                    .clone()
                    .try_into_match(
                        next_id,
                        account_and_conversation_id,
                        another_conversation_id,
                    )
                    .change_context(DieselDatabaseError::NotAllowed)?
            } else if interaction.is_match() {
                return Err(DieselDatabaseError::AlreadyDone.report());
            } else {
                let next_id = cmds
                    .read()
                    .chat()
                    .chat_state(id_like_receiver)?
                    .next_received_like_id;
                let updated_interaction = interaction
                    .clone()
                    .try_into_like(id_like_sender, id_like_receiver, next_id)
                    .change_context(DieselDatabaseError::NotAllowed)?;
                cmds.chat().modify_chat_state(id_like_receiver, |s| {
                    s.next_received_like_id = next_id.increment();
                })?;
                updated_interaction
            };
            cmds.chat()
                .interaction()
                .update_account_interaction(updated.clone())?;

            let sender = cmds.chat().modify_chat_state(id_like_sender, |_| ())?;

            let receiver = cmds.chat().modify_chat_state(id_like_receiver, |s| {
                if interaction.is_empty() {
                    if updated.included_in_received_new_likes_count {
                        s.new_received_likes_count = s.new_received_likes_count.increment();
                        s.received_likes_sync_version
                            .increment_if_not_max_value_mut();
                    }
                } else if interaction.is_like() && interaction.included_in_received_new_likes_count
                {
                    s.new_received_likes_count = s.new_received_likes_count.decrement();
                    s.received_likes_sync_version
                        .increment_if_not_max_value_mut();
                }
            })?;

            Ok(SenderAndReceiverStateChanges { sender, receiver })
        })
    }

    /// Delete a like.
    ///
    /// Returns Ok only if the state change happened.
    pub async fn delete_like(
        &self,
        id_sender: AccountIdInternal,
        id_receiver: AccountIdInternal,
    ) -> Result<SenderAndReceiverStateChanges, DataError> {
        db_transaction!(self, move |mut cmds| {
            let interaction = cmds
                .chat()
                .interaction()
                .get_or_create_account_interaction(id_sender, id_receiver)?;

            if interaction.is_empty() {
                return Err(DieselDatabaseError::AlreadyDone.report());
            }
            if !interaction.is_like() {
                return Err(DieselDatabaseError::NotAllowed.report());
            }
            if interaction.account_id_sender != Some(id_sender.into_db_id()) {
                return Err(DieselDatabaseError::NotAllowed.report());
            }
            let mut updated = interaction
                .clone()
                .try_into_empty()
                .change_context(DieselDatabaseError::NotAllowed)?;
            updated.set_previous_like_deleter_if_slot_available(id_sender);

            cmds.chat()
                .interaction()
                .update_account_interaction(updated)?;

            let sender = cmds.chat().modify_chat_state(id_sender, |_| ())?;

            let receiver = cmds.chat().modify_chat_state(id_receiver, |s| {
                s.received_likes_sync_version
                    .increment_if_not_max_value_mut();
                if interaction.included_in_received_new_likes_count {
                    s.new_received_likes_count = s.new_received_likes_count.decrement();
                    s.received_likes_sync_version
                        .increment_if_not_max_value_mut();
                }
            })?;

            Ok(SenderAndReceiverStateChanges { sender, receiver })
        })
    }

    /// Block a profile.
    ///
    /// Returns Ok only if the state change happened.
    pub async fn block_profile(
        &self,
        id_block_sender: AccountIdInternal,
        id_block_receiver: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let interaction = cmds
                .chat()
                .interaction()
                .get_or_create_account_interaction(id_block_sender, id_block_receiver)?;

            if interaction.is_direction_blocked(id_block_sender, id_block_receiver) {
                return Err(DieselDatabaseError::AlreadyDone.report());
            }
            let updated = interaction
                .clone()
                .add_block(id_block_sender, id_block_receiver);
            cmds.chat()
                .interaction()
                .update_account_interaction(updated)?;

            Ok(())
        })
    }

    /// Delete block.
    ///
    /// Returns Ok only if the state change happened.
    pub async fn delete_block(
        &self,
        id_block_sender: AccountIdInternal,
        id_block_receiver: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let interaction = cmds
                .chat()
                .interaction()
                .get_or_create_account_interaction(id_block_sender, id_block_receiver)?;

            if !interaction.is_direction_blocked(id_block_sender, id_block_receiver) {
                return Err(DieselDatabaseError::NotAllowed.report());
            }
            let updated = interaction
                .clone()
                .delete_block(id_block_sender, id_block_receiver);
            cmds.chat()
                .interaction()
                .update_account_interaction(updated)?;

            Ok(())
        })
    }

    // TODO(prod): Change SQLite settings that delete is overwriting.

    pub async fn add_receiver_acknowledgement_and_delete_if_also_sender_has_acknowledged(
        &self,
        message_receiver: AccountIdInternal,
        messages: Vec<PendingMessageId>,
    ) -> Result<(), DataError> {
        let mut converted = vec![];
        for m in messages {
            let sender = self.to_account_id_internal(m.sender).await?;
            converted.push(PendingMessageIdInternal {
                sender,
                receiver: message_receiver.into_db_id(),
                m: m.m,
            });
        }

        let pending_messages_exists = db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .message()
                .add_receiver_acknowledgement_and_delete_if_also_sender_has_acknowledged(
                    message_receiver,
                    converted,
                )?;

            cmds.read()
                .chat()
                .message()
                .pending_messages_exists(message_receiver)
        })?;

        if !pending_messages_exists {
            self.event_manager()
                .remove_specific_pending_notification_flags_from_cache(
                    message_receiver,
                    PendingNotificationFlags::NEW_MESSAGE,
                )
                .await;
        }

        Ok(())
    }

    pub async fn add_sender_acknowledgement_and_delete_if_also_receiver_has_acknowledged(
        &self,
        message_receiver: AccountIdInternal,
        messages: Vec<SentMessageId>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .message()
                .add_sender_acknowledgement_and_delete_if_also_receiver_has_acknowledged(
                    message_receiver,
                    messages,
                )
        })?;

        Ok(())
    }

    /// Insert a new pending message if sender and receiver are a match and
    /// one or two way block exists.
    ///
    /// Receiver public key check is for preventing client from
    /// sending messages encrypted with outdated public key.
    ///
    /// Max receiver acknowledgements missing count is 50.
    ///
    /// Max sender acknowledgements missing count is 50.
    ///
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_pending_message_if_match_and_not_blocked(
        &self,
        sender: AccountIdInternal,
        receiver: AccountIdInternal,
        message: Vec<u8>,
        sender_public_key_from_client: PublicKeyId,
        receiver_public_key_from_client: PublicKeyId,
        client_id_value: ClientId,
        client_local_id_value: ClientLocalId,
        keys: Arc<ParsedKeys>,
    ) -> Result<(SendMessageResult, Option<PushNotificationAllowed>), DataError> {
        db_transaction!(self, move |mut cmds| {
            let sender_current_key = cmds
                .read()
                .chat()
                .public_key()
                .latest_public_key_id(sender)?;
            if Some(sender_public_key_from_client) != sender_current_key {
                return Ok((SendMessageResult::sender_public_key_outdated(), None));
            }

            let receiver_current_key = cmds
                .read()
                .chat()
                .public_key()
                .latest_public_key_id(receiver)?;
            if Some(receiver_public_key_from_client) != receiver_current_key {
                return Ok((SendMessageResult::receiver_public_key_outdated(), None));
            }

            let receiver_acknowledgements_missing = cmds
                .read()
                .chat()
                .message()
                .receiver_acknowledgements_missing_count_for_one_conversation(sender, receiver)?;

            if receiver_acknowledgements_missing >= 50 {
                return Ok((
                    SendMessageResult::too_many_receiver_acknowledgements_missing(),
                    None,
                ));
            }

            let sender_acknowledgements_missing = cmds
                .read()
                .chat()
                .message()
                .sender_acknowledgements_missing_count_for_one_conversation(sender, receiver)?;

            if sender_acknowledgements_missing >= 50 {
                return Ok((
                    SendMessageResult::too_many_sender_acknowledgements_missing(),
                    None,
                ));
            }

            let message_values = cmds
                .chat()
                .message()
                .insert_pending_message_if_match_and_not_blocked(
                    sender,
                    receiver,
                    sender_public_key_from_client,
                    receiver_public_key_from_client,
                    message,
                    client_id_value,
                    client_local_id_value,
                    keys,
                )?;

            let message_values = match message_values {
                Ok(v) => v,
                Err(ReceiverBlockedSender) => {
                    return Ok((
                        SendMessageResult::receiver_blocked_sender_or_receiver_not_found(),
                        None,
                    ));
                }
            };

            let push_notification_allowd = if receiver_acknowledgements_missing <= 1 {
                // Update new message notification twice so that notification
                // displays singular or plural text correctly.
                Some(PushNotificationAllowed)
            } else {
                None
            };

            Ok((
                SendMessageResult::successful(message_values),
                push_notification_allowd,
            ))
        })
    }

    pub async fn add_public_key(
        &self,
        id: AccountIdInternal,
        new_key: Vec<u8>,
    ) -> Result<AddPublicKeyResult, DataError> {
        let info = self
            .handle()
            .read()
            .chat()
            .public_key()
            .get_private_public_key_info(id)
            .await?;

        let key_count = if let Some(id) = info.latest_public_key_id {
            if *id.as_i64() >= 0 && *id.as_i64() < i64::MAX {
                *id.as_i64() + 1
            } else {
                return Err(DataError::NotAllowed.report().into());
            }
        } else {
            0
        };

        if key_count >= info.public_key_count_limit() {
            return Ok(AddPublicKeyResult::error_too_many_keys());
        }

        let key = db_transaction!(self, move |mut cmds| {
            cmds.chat().add_public_key(id, new_key)
        })?;

        Ok(AddPublicKeyResult::success(key))
    }

    /// Resets new received likes count if needed and updates received likes
    /// iterator reset time.
    pub async fn handle_reset_received_likes_iterator(
        &self,
        id: AccountIdInternal,
    ) -> Result<
        (
            ReceivedLikesIteratorSessionIdInternal,
            ReceivedLikesSyncVersion,
        ),
        DataError,
    > {
        let (new_version, received_like_id, received_like_id_previous) =
            db_transaction!(self, move |mut cmds| {
                cmds.chat()
                    .interaction()
                    .reset_included_in_received_new_likes_count(id)?;
                let state = cmds.read().chat().chat_state(id)?;
                let latest_used_id = state.next_received_like_id.next_id_to_latest_used_id();
                let id_at_previous_reset = state.received_like_id_at_received_likes_iterator_reset;
                cmds.chat().modify_chat_state(id, |s| {
                    if s.new_received_likes_count.c != 0 {
                        s.received_likes_sync_version
                            .increment_if_not_max_value_mut();
                        s.new_received_likes_count = NewReceivedLikesCount::default();
                    }
                    s.received_like_id_at_received_likes_iterator_reset = Some(latest_used_id);
                })?;
                let new_state = cmds.read().chat().chat_state(id)?;
                Ok((
                    new_state.received_likes_sync_version,
                    latest_used_id,
                    id_at_previous_reset,
                ))
            })?;

        let session_id = self
            .write_cache_chat(id.as_id(), |e| {
                Ok(e.received_likes_iterator
                    .reset(received_like_id, received_like_id_previous))
            })
            .await
            .into_data_error(id)?;

        Ok((session_id, new_version))
    }

    pub async fn handle_reset_matches_iterator(
        &self,
        id: AccountIdInternal,
    ) -> Result<MatchesIteratorSessionIdInternal, DataError> {
        let latest_used_id = self
            .db_read(|mut cmds| cmds.chat().global_state())
            .await?
            .next_match_id
            .next_id_to_latest_used_id();
        let session_id = self
            .write_cache_chat(id.as_id(), |e| Ok(e.matches_iterator.reset(latest_used_id)))
            .await
            .into_data_error(id)?;

        Ok(session_id)
    }
}

pub struct SenderAndReceiverStateChanges {
    pub sender: ChatStateChanges,
    pub receiver: ChatStateChanges,
}

/// Message push notification is allowed to be sent if receiver side
/// of acknowledgement queue is empty when sending a new message.
/// This avoids sending multiple push notifications if client is running
/// in background and can receive push notifications.
pub struct PushNotificationAllowed;
