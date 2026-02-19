mod limits;
mod notification;
mod privacy;
mod report;
mod transfer;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use database_chat::current::{
    read::GetDbReadCommandsChat,
    write::{
        GetDbWriteCommandsChat,
        chat::{ChatStateChanges, ReceiverBlockedSender},
    },
};
use error_stack::ResultExt;
use model::{NewReceivedLikesCountResult, ReceivedLikeId};
use model_chat::{
    AccountIdInternal, AddPublicKeyResult, ChatStateRaw, DeliveryInfoType, LatestSeenMessageInfo,
    MessageId, NewReceivedLikesCount, PendingMessageId, PublicKeyId, ReceivedLikesIteratorState,
    ResetReceivedLikesIteratorResult, SeenMessage, SendMessageResult, SyncVersionUtils,
};
use server_data::{
    DataError, DieselDatabaseError, app::GetConfig, db_transaction, define_cmd_wrapper_write,
    id::ToAccountIdInternal, read::DbRead, result::Result, write::DbTransaction,
};
use simple_backend_utils::ContextExt;
use utils::encrypt::ParsedKeys;

use crate::read::GetReadChatCommands;

define_cmd_wrapper_write!(WriteCommandsChat);

impl<'a> WriteCommandsChat<'a> {
    pub fn report(self) -> report::WriteCommandsChatReport<'a> {
        report::WriteCommandsChatReport::new(self.0)
    }
    pub fn notification(self) -> notification::WriteCommandsChatNotification<'a> {
        notification::WriteCommandsChatNotification::new(self.0)
    }
    pub fn privacy(self) -> privacy::WriteCommandsChatPrivacy<'a> {
        privacy::WriteCommandsChatPrivacy::new(self.0)
    }
    pub fn limits(self) -> limits::WriteCommandsChatLimits<'a> {
        limits::WriteCommandsChatLimits::new(self.0)
    }
    pub fn transfer(self) -> transfer::WriteCommandsChatTransfer<'a> {
        transfer::WriteCommandsChatTransfer::new(self.0)
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
                let conversation_id_receiver =
                    cmds.chat().upsert_next_conversation_id(id_like_receiver)?;
                cmds.chat().message().upsert_conversation_id(
                    id_like_receiver,
                    id_like_sender,
                    conversation_id_receiver,
                )?;
                let conversation_id_sender =
                    cmds.chat().upsert_next_conversation_id(id_like_sender)?;
                cmds.chat().message().upsert_conversation_id(
                    id_like_sender,
                    id_like_receiver,
                    conversation_id_sender,
                )?;

                let next_id = cmds.chat().upsert_next_match_id()?;
                interaction
                    .clone()
                    .try_into_match(next_id)
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
                    s.new_received_likes_count = s.new_received_likes_count.increment();
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

    pub async fn add_receiver_acknowledgement_and_delete_if_also_sender_has_acknowledged(
        &self,
        message_receiver: AccountIdInternal,
        messages: Vec<PendingMessageId>,
        delivery_failed: bool,
    ) -> Result<(), DataError> {
        let mut converted = vec![];
        let mut unique_senders = HashSet::new();
        for m in messages {
            let sender = self.to_account_id_internal(m.sender).await?;
            let msg_info = self
                .db_read(move |mut cmds| {
                    cmds.chat()
                        .message()
                        .check_pending_message_info(sender, message_receiver, m.id)
                })
                .await?;
            if let Some(msg_info) = msg_info {
                converted.push(msg_info);
                unique_senders.insert(sender);
            }
        }

        let delivery_type = if delivery_failed {
            DeliveryInfoType::DeliveryFailed
        } else {
            DeliveryInfoType::Delivered
        };

        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .message()
                .add_receiver_acknowledgement_and_delete_if_also_sender_has_acknowledged(
                    message_receiver,
                    &converted,
                )?;

            for msg in &converted {
                cmds.chat().message().insert_message_delivery_info(
                    msg.sender,
                    message_receiver,
                    msg.message_id,
                    delivery_type,
                )?;
            }

            Ok(())
        })?;

        for sender in &unique_senders {
            self.handle()
                .events()
                .send_connected_event(
                    sender.as_id(),
                    model::EventToClientInternal::MessageDeliveryInfoChanged,
                )
                .await?;
        }

        Ok(())
    }

    pub async fn mark_message_as_seen(
        &self,
        message_receiver: AccountIdInternal,
        message: SeenMessage,
    ) -> Result<(), DataError> {
        let seen_state_enabled = self
            .config()
            .client_features_internal()
            .chat
            .message_state_seen;

        if !seen_state_enabled {
            return Ok(());
        }

        let sender = self.to_account_id_internal(message.sender).await?;

        let sender_was_updated = db_transaction!(self, move |mut cmds| {
            let Some(message_number_max) = cmds
                .read()
                .chat()
                .interaction()
                .account_interaction(message_receiver, sender)?
                .map(|v| v.next_message_number().mn - 1)
            else {
                return Ok(false);
            };

            if message.mn.mn > message_number_max {
                // message is not sent yet
                return Ok(false);
            }

            let message_number_min = cmds
                .read()
                .chat()
                .message()
                .get_latest_seen_message_number(message_receiver, sender)?
                .map(|v| v.mn);

            if let Some(message_number_min) = message_number_min
                && message.mn.mn <= message_number_min
            {
                // message is already marked as seen
                return Ok(false);
            }

            cmds.chat().message().update_latest_seen_message(
                message_receiver,
                sender,
                message.mn,
            )?;

            Ok(true)
        })?;

        if sender_was_updated {
            self.handle()
                .events()
                .send_connected_event(
                    sender.as_id(),
                    model::EventToClientInternal::LatestSeenMessageChanged,
                )
                .await?;
        }

        Ok(())
    }

    pub async fn delete_delivery_info_by_ids(
        &self,
        sender_id: AccountIdInternal,
        ids: Vec<i64>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .message()
                .delete_delivery_info_by_ids(sender_id, ids)
        })?;

        Ok(())
    }

    pub async fn delete_pending_latest_seen_message_deliveries(
        &self,
        sender_id: AccountIdInternal,
        acknowledged: Vec<LatestSeenMessageInfo>,
    ) -> Result<(), DataError> {
        let mut acknowledged_internal: HashMap<AccountIdInternal, model::MessageNumber> =
            HashMap::new();
        for info in acknowledged {
            let viewer = self.to_account_id_internal(info.viewer).await?;
            acknowledged_internal
                .entry(viewer)
                .and_modify(|current| {
                    if current.mn < info.mn.mn {
                        *current = info.mn;
                    }
                })
                .or_insert(info.mn);
        }

        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .message()
                .delete_pending_latest_seen_message_deliveries(sender_id, acknowledged_internal)
        })?;

        Ok(())
    }

    pub async fn add_sender_acknowledgement_and_delete_if_also_receiver_has_acknowledged(
        &self,
        message_receiver: AccountIdInternal,
        messages: Vec<MessageId>,
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
    #[allow(clippy::too_many_arguments)]
    pub async fn insert_pending_message_if_match_and_not_blocked(
        &self,
        sender: AccountIdInternal,
        receiver: AccountIdInternal,
        message: Vec<u8>,
        sender_public_key_from_client: PublicKeyId,
        receiver_public_key_from_client: PublicKeyId,
        message_id_value: MessageId,
        keys: Arc<ParsedKeys>,
    ) -> Result<(SendMessageResult, Option<PushNotificationAllowed>), DataError> {
        let chat_limits = self.config().limits_chat();
        let conversation_pending_messages_max_count =
            i64::from(chat_limits.conversation_pending_messages_max_count);

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

            let sender_has_unreceived_delivery_info = cmds
                .read()
                .chat()
                .message()
                .has_unreceived_delivery_info(sender)?;

            if sender_has_unreceived_delivery_info {
                return Ok((SendMessageResult::pending_delivery_info_exists(), None));
            }

            let receiver_acknowledgements_missing = cmds
                .read()
                .chat()
                .message()
                .receiver_acknowledgements_missing_count_for_one_conversation(sender, receiver)?;

            if receiver_acknowledgements_missing >= conversation_pending_messages_max_count {
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

            if sender_acknowledgements_missing >= conversation_pending_messages_max_count {
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
                    message_id_value,
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

            let remaining_conversation_messages = {
                let missing_acknowledgements = i64::max(
                    receiver_acknowledgements_missing,
                    sender_acknowledgements_missing,
                );
                let remaining = conversation_pending_messages_max_count
                    .saturating_sub(missing_acknowledgements);
                if remaining <= 0 {
                    Some(0)
                } else if remaining <= 5 {
                    Some(remaining as u16)
                } else {
                    None
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
                SendMessageResult::successful(message_values, remaining_conversation_messages),
                push_notification_allowd,
            ))
        })
    }

    pub async fn add_public_key(
        &self,
        id: AccountIdInternal,
        new_key: Vec<u8>,
        ignore_pending_messages: bool,
    ) -> Result<AddPublicKeyResult, DataError> {
        let info = self
            .handle()
            .read()
            .chat()
            .public_key()
            .get_private_public_key_info(id)
            .await?;

        let key_count = if let Some(id) = info.latest_public_key_id {
            if *id.as_ref() >= 0 && *id.as_ref() < i64::MAX {
                *id.as_ref() + 1
            } else {
                return Err(DataError::NotAllowed.report().into());
            }
        } else {
            0
        };

        if key_count >= info.public_key_count_limit() {
            return Ok(AddPublicKeyResult::error_too_many_keys());
        }

        if !ignore_pending_messages {
            let pending_messages = self.handle().read().chat().all_pending_messages(id).await?;

            if !pending_messages.is_empty() {
                return Ok(AddPublicKeyResult::error_pending_messages_found());
            }
        }

        let key = db_transaction!(self, move |mut cmds| {
            cmds.chat().add_public_key(id, new_key)
        })?;

        Ok(AddPublicKeyResult::success(key))
    }

    /// Resets new received likes count if needed
    pub async fn handle_reset_new_received_likes_count(
        &self,
        id: AccountIdInternal,
    ) -> Result<NewReceivedLikesCountResult, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().modify_chat_state(id, |s| {
                if s.new_received_likes_count.c != 0 {
                    s.received_likes_sync_version
                        .increment_if_not_max_value_mut();
                    s.new_received_likes_count = NewReceivedLikesCount::default();
                }
            })?;
            let new_state = cmds.read().chat().chat_state(id)?;
            Ok(new_state.new_received_likes_info())
        })
    }

    pub async fn handle_reset_received_likes_iterator(
        &self,
        id: AccountIdInternal,
    ) -> Result<ResetReceivedLikesIteratorResult, DataError> {
        db_transaction!(self, move |mut cmds| {
            let state = cmds.read().chat().chat_state(id)?;
            let id_at_reset = state.next_received_like_id.next_id_to_latest_used_id();
            Ok(ResetReceivedLikesIteratorResult {
                s: ReceivedLikesIteratorState {
                    id_at_reset,
                    page: 0,
                },
            })
        })
    }

    pub async fn mark_received_likes_viewed(
        &self,
        like_receiver: AccountIdInternal,
        likes: Vec<ReceivedLikeId>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .interaction()
                .mark_received_likes_viewed(like_receiver, likes)?;
            Ok(())
        })
    }

    /// Mark video call URL as created for the caller account.
    ///
    /// This determines whether the caller is the sender or receiver in the
    /// interaction and sets the appropriate flag.
    pub async fn mark_video_call_url_created(
        &self,
        caller: AccountIdInternal,
        other_user: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let mut interaction = cmds
                .read()
                .chat()
                .interaction()
                .account_interaction(caller, other_user)?
                .ok_or(DieselDatabaseError::NotAllowed)?;

            // Determine if caller is like sender or receiver and set appropriate flag
            if interaction.account_id_sender == Some(caller.into_db_id()) {
                interaction.video_call_url_created_sender = true;
            } else {
                interaction.video_call_url_created_receiver = true;
            }

            cmds.chat()
                .interaction()
                .update_account_interaction(interaction)?;

            Ok(())
        })
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
