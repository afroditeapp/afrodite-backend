mod push_notifications;

use database_chat::current::write::chat::{ChatStateChanges, ReceiverBlockedSender};
use error_stack::ResultExt;
use model::{AccountIdInternal, ChatStateRaw, ClientId, ClientLocalId, MatchesIteratorSessionIdInternal, MessageNumber, NewReceivedLikesCount, PendingMessageId, PendingMessageIdInternal, PendingNotificationFlags, PublicKeyId, PublicKeyVersion, ReceivedLikesIteratorSessionIdInternal, ReceivedLikesSyncVersion, SendMessageResult, SentMessageId, SetPublicKey, SyncVersionUtils};
use server_data::{
    cache::{chat::limit::ChatLimits, CacheError}, define_server_data_write_commands, result::Result, write::WriteCommandsProvider, DataError, DieselDatabaseError, IntoDataError
};
use simple_backend_utils::ContextExt;

use self::push_notifications::WriteCommandsChatPushNotifications;

define_server_data_write_commands!(WriteCommandsChat);
define_db_transaction_command!(WriteCommandsChat);
define_db_read_command_for_write!(WriteCommandsChat);

impl<C: WriteCommandsProvider> WriteCommandsChat<C> {
    pub fn push_notifications(self) -> WriteCommandsChatPushNotifications<C> {
        WriteCommandsChatPushNotifications::new(self.cmds)
    }
}

impl<C: WriteCommandsProvider> WriteCommandsChat<C> {
    pub async fn modify_chat_limits<T>(
        &mut self,
        id: AccountIdInternal,
        mut action: impl FnMut(&mut ChatLimits) -> T,
    ) -> Result<T, DataError> {
        let value = self.cache().write_cache(id, move |entry| {
            let chat = entry.chat_data_mut()?;
            Ok(action(&mut chat.limits))
        })
            .await?;

        Ok(value)
    }

    pub async fn modify_chat_state(
        &mut self,
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
        &mut self,
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
                let next_id = cmds.read().chat().global_state()?.next_match_id;
                let updated_interaction = interaction
                    .clone()
                    .try_into_match(next_id)
                    .change_context(DieselDatabaseError::NotAllowed)?;
                cmds.chat().upsert_next_match_id()?;
                updated_interaction
            } else if interaction.is_match() {
                return Err(DieselDatabaseError::AlreadyDone.report());
            } else {
                let next_id = cmds.read().chat().chat_state(id_like_receiver)?.next_received_like_id;
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

            let sender = cmds.chat().modify_chat_state(id_like_sender, |s| {
                if interaction.is_empty() {
                    s.sent_likes_sync_version.increment_if_not_max_value_mut();
                } else if interaction.is_like() {
                    s.matches_sync_version.increment_if_not_max_value_mut();
                }
            })?;

            let receiver = cmds.chat().modify_chat_state(id_like_receiver, |s| {
                if interaction.is_empty() {
                    if updated.included_in_received_new_likes_count {
                        s.new_received_likes_count = s.new_received_likes_count.increment();
                        s.received_likes_sync_version
                            .increment_if_not_max_value_mut();
                    }
                } else if interaction.is_like() {
                    s.matches_sync_version.increment_if_not_max_value_mut();

                    if interaction.included_in_received_new_likes_count {
                        s.new_received_likes_count = s.new_received_likes_count.decrement();
                        s.received_likes_sync_version
                            .increment_if_not_max_value_mut();
                    }
                }
            })?;

            Ok(SenderAndReceiverStateChanges { sender, receiver })
        })
    }

    /// Delete a like.
    ///
    /// Returns Ok only if the state change happened.
    pub async fn delete_like(
        &mut self,
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

            let sender = cmds.chat().modify_chat_state(id_sender, |s| {
                s.sent_likes_sync_version.increment_if_not_max_value_mut();
            })?;

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
        &mut self,
        id_block_sender: AccountIdInternal,
        id_block_receiver: AccountIdInternal,
    ) -> Result<SenderAndReceiverStateChanges, DataError> {
        db_transaction!(self, move |mut cmds| {
            let interaction = cmds
                .chat()
                .interaction()
                .get_or_create_account_interaction(id_block_sender, id_block_receiver)?;

            if interaction.is_direction_blocked(
                id_block_sender,
                id_block_receiver,
            ) {
                return Err(DieselDatabaseError::AlreadyDone.report());
            }
            let updated = interaction
                .clone()
                .add_block(id_block_sender, id_block_receiver);
            cmds.chat()
                .interaction()
                .update_account_interaction(updated)?;

            let sender = cmds.chat().modify_chat_state(id_block_sender, |s| {
                s.sent_blocks_sync_version.increment_if_not_max_value_mut();
            })?;

            let receiver = cmds.chat().modify_chat_state(id_block_receiver, |s| {
                s.received_blocks_sync_version
                    .increment_if_not_max_value_mut();
            })?;

            Ok(SenderAndReceiverStateChanges { sender, receiver })
        })
    }

    /// Delete block.
    ///
    /// Returns Ok only if the state change happened.
    pub async fn delete_block(
        &mut self,
        id_block_sender: AccountIdInternal,
        id_block_receiver: AccountIdInternal,
    ) -> Result<SenderAndReceiverStateChanges, DataError> {
        db_transaction!(self, move |mut cmds| {
            let interaction = cmds
                .chat()
                .interaction()
                .get_or_create_account_interaction(id_block_sender, id_block_receiver)?;

            if !interaction.is_direction_blocked(
                id_block_sender,
                id_block_receiver,
            ) {
                return Err(DieselDatabaseError::NotAllowed.report());
            }
            let updated = interaction
                .clone()
                .delete_block(id_block_sender, id_block_receiver);
            cmds.chat()
                .interaction()
                .update_account_interaction(updated)?;

            let sender = cmds.chat().modify_chat_state(id_block_sender, |s| {
                s.sent_blocks_sync_version.increment_if_not_max_value_mut();
            })?;

            let receiver = cmds.chat().modify_chat_state(id_block_receiver, |s| {
                s.received_blocks_sync_version
                    .increment_if_not_max_value_mut();
            })?;

            Ok(SenderAndReceiverStateChanges { sender, receiver })
        })
    }

    // TODO(prod): Change SQLite settings that delete is overwriting.

    pub async fn add_receiver_acknowledgement_and_delete_if_also_sender_has_acknowledged(
        &mut self,
        message_receiver: AccountIdInternal,
        messages: Vec<PendingMessageId>,
    ) -> Result<(), DataError> {
        let mut converted = vec![];
        for m in messages {
            let sender = self.cache().to_account_id_internal(m.sender).await?;
            converted.push(PendingMessageIdInternal {
                sender,
                mn: m.mn,
            });
        }

        let pending_messages = db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .message()
                .add_receiver_acknowledgement_and_delete_if_also_sender_has_acknowledged(message_receiver, converted)?;

            cmds.read().chat().message().all_pending_message_sender_account_ids(message_receiver)
        })?;

        if pending_messages.is_empty() {
            self
                .events()
                .remove_specific_pending_notification_flags_from_cache(message_receiver, PendingNotificationFlags::NEW_MESSAGE)
                .await;
        }

        Ok(())
    }

    pub async fn add_sender_acknowledgement_and_delete_if_also_receiver_has_acknowledged(
        &mut self,
        message_receiver: AccountIdInternal,
        messages: Vec<SentMessageId>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .message()
                .add_sender_acknowledgement_and_delete_if_also_receiver_has_acknowledged(message_receiver, messages)
        })?;

        Ok(())
    }

    /// Update message number which my account has viewed from the sender
    pub async fn update_message_number_of_latest_viewed_message(
        &self,
        id_my_account: AccountIdInternal,
        id_message_sender: AccountIdInternal,
        new_message_number: MessageNumber,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let mut interaction = cmds
                .read()
                .chat()
                .interaction()
                .account_interaction(id_my_account, id_message_sender)?
                .ok_or(DieselDatabaseError::NotFound.report())?;

            // Prevent marking future messages as viewed
            if new_message_number.mn > interaction.message_counter {
                return Err(DieselDatabaseError::NotAllowed.report());
            }

            // Who is sender and receiver in the interaction data depends
            // on who did the first like
            let modify_number = if interaction.account_id_sender == Some(id_my_account.into_db_id())
            {
                interaction.sender_latest_viewed_message.as_mut()
            } else {
                interaction.receiver_latest_viewed_message.as_mut()
            };

            if let Some(number) = modify_number {
                *number = new_message_number;
            } else {
                return Err(DieselDatabaseError::NotAllowed.report());
            }

            cmds.chat()
                .interaction()
                .update_account_interaction(interaction)?;

            Ok(())
        })
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
        &mut self,
        sender: AccountIdInternal,
        receiver: AccountIdInternal,
        message: Vec<u8>,
        receiver_public_key_from_client: PublicKeyId,
        receiver_public_key_version_from_client: PublicKeyVersion,
        client_id_value: ClientId,
        client_local_id_value: ClientLocalId,
    ) -> Result<(SendMessageResult, Option<PushNotificationAllowed>), DataError> {
        db_transaction!(self, move |mut cmds| {
            let current_key = cmds.read().chat().public_key(
                receiver,
                receiver_public_key_version_from_client
            )?;
            if Some(receiver_public_key_from_client) != current_key.map(|v| v.id) {
                return Ok((SendMessageResult::public_key_outdated(), None));
            }

            let receiver_acknowledgements_missing = cmds
                .read()
                .chat()
                .message()
                .receiver_acknowledgements_missing_count_for_one_conversation(sender, receiver)?;

            if receiver_acknowledgements_missing >= 50 {
                return Ok((SendMessageResult::too_many_receiver_acknowledgements_missing(), None));
            }

            let sender_acknowledgements_missing = cmds
                .read()
                .chat()
                .message()
                .sender_acknowledgements_missing_count_for_one_conversation(sender, receiver)?;

            if sender_acknowledgements_missing >= 50 {
                return Ok((SendMessageResult::too_many_sender_acknowledgements_missing(), None));
            }

            let message_values = cmds.chat()
                .message()
                .insert_pending_message_if_match_and_not_blocked(
                    sender,
                    receiver,
                    message,
                    client_id_value,
                    client_local_id_value,
                )?;

            let message_values = match message_values {
                Ok(v) => v,
                Err(ReceiverBlockedSender) =>
                    return Ok((SendMessageResult::receiver_blocked_sender_or_receiver_not_found(), None)),
            };

            let push_notification_allowd = if receiver_acknowledgements_missing == 0 {
                Some(PushNotificationAllowed)
            } else {
                None
            };

            Ok((SendMessageResult::successful(message_values), push_notification_allowd))
        })
    }

    pub async fn set_public_key(
        &mut self,
        id: AccountIdInternal,
        data: SetPublicKey,
    ) -> Result<PublicKeyId, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .set_public_key(id, data)
        })
    }

    /// Resets new received likes count if needed and updates received likes
    /// iterator reset time.
    pub async fn handle_reset_received_likes_iterator(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(ReceivedLikesIteratorSessionIdInternal, ReceivedLikesSyncVersion), DataError> {
        let (new_version, received_like_id, received_like_id_previous) = db_transaction!(self, move |mut cmds| {
            cmds.chat().interaction().reset_included_in_received_new_likes_count(id)?;
            let state = cmds.read().chat().chat_state(id)?;
            let latest_used_id = state.next_received_like_id.next_id_to_latest_used_id();
            let id_at_previous_reset = state.received_like_id_at_received_likes_iterator_reset;
            cmds.chat().modify_chat_state(id, |s| {
                if s.new_received_likes_count.c != 0 {
                    s.received_likes_sync_version.increment_if_not_max_value_mut();
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

        let session_id = self.cache()
            .write_cache(id.as_id(), |e| {
                if let Some(c) = e.chat.as_mut() {
                    Ok(c.received_likes_iterator.reset(received_like_id, received_like_id_previous))
                } else {
                    Err(CacheError::FeatureNotEnabled.report())
                }
            })
            .await
            .into_data_error(id)?;

        Ok((session_id, new_version))
    }

    pub async fn handle_reset_matches_iterator(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<MatchesIteratorSessionIdInternal, DataError> {
        let latest_used_id = self.db_read(|mut cmds| cmds.chat().global_state()).await?
            .next_match_id
            .next_id_to_latest_used_id();
        let session_id = self.cache()
            .write_cache(id.as_id(), |e| {
                if let Some(c) = e.chat.as_mut() {
                    Ok(c.matches_iterator.reset(latest_used_id))
                } else {
                    Err(CacheError::FeatureNotEnabled.report())
                }
            })
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
