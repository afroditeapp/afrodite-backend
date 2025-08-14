use std::collections::HashMap;

use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountId, AccountIdDb, AdminDataExportPendingMessage, ConversationId,
    DataExportPendingMessage, MessageId, NewMessageNotification, NewMessageNotificationList,
    PendingMessageIdInternal, PendingMessageIdInternalAndMessageTime, PendingMessageRaw, UnixTime,
};
use model_chat::{AccountIdInternal, GetSentMessage, PendingMessageInternal, SentMessageId};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadChatMessage);

impl CurrentReadChatMessage<'_> {
    pub fn all_pending_messages(
        &mut self,
        id_message_receiver: AccountIdInternal,
    ) -> Result<Vec<Vec<u8>>, DieselDatabaseError> {
        use crate::schema::pending_messages::dsl::*;

        let value: Vec<Vec<u8>> = pending_messages
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .filter(receiver_acknowledgement.eq(false))
            .select(message_bytes)
            .load(self.conn())
            .into_db_error(())?;

        Ok(value)
    }

    pub fn pending_messages_exists(
        &mut self,
        id_message_receiver: AccountIdInternal,
    ) -> Result<bool, DieselDatabaseError> {
        use crate::schema::pending_messages::dsl::*;

        let value: Option<i64> = pending_messages
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .filter(receiver_acknowledgement.eq(false))
            .select(id)
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(value.is_some())
    }

    pub fn new_message_notification_list(
        &mut self,
        id_message_receiver: AccountIdInternal,
    ) -> Result<(NewMessageNotificationList, Vec<PendingMessageIdInternal>), DieselDatabaseError>
    {
        use crate::schema::{account_id, account_interaction, pending_messages::dsl::*};

        let data: Vec<(
            AccountIdDb,
            AccountId,
            AccountIdDb,
            ConversationId,
            ConversationId,
            bool,
            MessageId,
        )> = pending_messages
            .inner_join(account_id::table.on(account_id_sender.eq(account_id::id)))
            .inner_join(account_interaction::table)
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .filter(receiver_acknowledgement.eq(false))
            .filter(account_interaction::account_id_sender.is_not_null())
            .filter(account_interaction::conversation_id_sender.is_not_null())
            .filter(account_interaction::conversation_id_receiver.is_not_null())
            .select((
                account_id::id,
                account_id::uuid,
                account_interaction::account_id_sender.assume_not_null(),
                account_interaction::conversation_id_sender.assume_not_null(),
                account_interaction::conversation_id_receiver.assume_not_null(),
                receiver_push_notification_sent,
                message_id,
            ))
            .order_by(account_id_sender)
            .load(self.conn())
            .into_db_error(())?;

        let mut notifications = HashMap::<AccountIdDb, (NewMessageNotification, bool)>::new();
        let mut messages_pending_push_notification = vec![];

        for (
            sender_db_id,
            sender,
            like_sender_db_id,
            conversation_id_sender,
            conversation_id_receiver,
            push_notification_sent,
            message_id_value,
        ) in data
        {
            // Select message receiver specific conversation ID
            let c = if like_sender_db_id == id_message_receiver.into_db_id() {
                conversation_id_sender
            } else {
                conversation_id_receiver
            };
            let mut entry = notifications
                .entry(sender_db_id)
                .insert_entry((NewMessageNotification { a: sender, c, m: 0 }, true));
            entry.get_mut().0.m += 1;

            if !push_notification_sent {
                // Message notification needs an update
                entry.get_mut().1 = false;
                messages_pending_push_notification.push(PendingMessageIdInternal {
                    sender: AccountIdInternal {
                        id: sender_db_id,
                        uuid: sender,
                    },
                    receiver: id_message_receiver.into_db_id(),
                    m: message_id_value,
                });
            }
        }

        let v = notifications
            .into_values()
            .filter_map(|(n, no_update)| if no_update { None } else { Some(n) })
            .collect();

        Ok((
            NewMessageNotificationList { v },
            messages_pending_push_notification,
        ))
    }

    pub fn messages_without_sent_email_notification(
        &mut self,
        id_message_receiver: AccountIdInternal,
    ) -> Result<Vec<PendingMessageIdInternalAndMessageTime>, DieselDatabaseError> {
        use crate::schema::{account_id, pending_messages::dsl::*};

        let data: Vec<(AccountIdDb, AccountId, AccountIdDb, UnixTime, MessageId)> =
            pending_messages
                .inner_join(account_id::table.on(account_id_sender.eq(account_id::id)))
                .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
                .filter(receiver_acknowledgement.eq(false))
                .filter(receiver_email_notification_sent.eq(false))
                .select((
                    account_id::id,
                    account_id::uuid,
                    account_id_receiver,
                    message_unix_time,
                    message_id,
                ))
                .order_by(account_id_sender)
                .load(self.conn())
                .into_db_error(())?;

        let v = data
            .into_iter()
            .map(|(sender_db_id, sender_id, receiver_db_id, time, m)| {
                PendingMessageIdInternalAndMessageTime {
                    id: PendingMessageIdInternal {
                        sender: AccountIdInternal {
                            id: sender_db_id,
                            uuid: sender_id,
                        },
                        receiver: receiver_db_id,
                        m,
                    },
                    time,
                }
            })
            .collect();

        Ok(v)
    }

    pub fn all_sent_messages(
        &mut self,
        id_message_sender: AccountIdInternal,
    ) -> Result<Vec<SentMessageId>, DieselDatabaseError> {
        use crate::schema::pending_messages::dsl::*;

        let value: Vec<PendingMessageInternal> = pending_messages
            .filter(account_id_sender.eq(id_message_sender.as_db_id()))
            .filter(sender_acknowledgement.eq(false))
            .select(PendingMessageInternal::as_select())
            .load(self.conn())
            .into_db_error(())?;

        let messages = value
            .into_iter()
            .map(|msg| SentMessageId {
                c: msg.sender_client_id,
                l: msg.sender_client_local_id,
            })
            .collect();

        Ok(messages)
    }

    pub fn get_sent_message(
        &mut self,
        id_message_sender: AccountIdInternal,
        message: SentMessageId,
    ) -> Result<GetSentMessage, DieselDatabaseError> {
        use crate::schema::pending_messages::dsl::*;

        let value: Vec<u8> = pending_messages
            .filter(account_id_sender.eq(id_message_sender.as_db_id()))
            .filter(sender_acknowledgement.eq(false))
            .filter(sender_client_id.eq(message.c))
            .filter(sender_client_local_id.eq(message.l))
            .select(message_bytes)
            .first(self.conn())
            .into_db_error(())?;

        Ok(GetSentMessage::new(value))
    }

    pub fn receiver_acknowledgements_missing_count_for_one_conversation(
        &mut self,
        id_message_sender: AccountIdInternal,
        id_message_receiver: AccountIdInternal,
    ) -> Result<i64, DieselDatabaseError> {
        use crate::schema::pending_messages::dsl::*;

        pending_messages
            .filter(account_id_sender.eq(id_message_sender.as_db_id()))
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .filter(receiver_acknowledgement.eq(false))
            .count()
            .get_result(self.conn())
            .into_db_error(())
    }

    pub fn sender_acknowledgements_missing_count_for_one_conversation(
        &mut self,
        id_message_sender: AccountIdInternal,
        id_message_receiver: AccountIdInternal,
    ) -> Result<i64, DieselDatabaseError> {
        use crate::schema::pending_messages::dsl::*;

        pending_messages
            .filter(account_id_sender.eq(id_message_sender.as_db_id()))
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .filter(sender_acknowledgement.eq(false))
            .count()
            .get_result(self.conn())
            .into_db_error(())
    }

    fn data_export_pending_messages_internal(
        &mut self,
        id_sender: AccountIdInternal,
    ) -> Result<Vec<(PendingMessageRaw, Vec<u8>)>, DieselDatabaseError> {
        use crate::schema::pending_messages::dsl::*;

        pending_messages
            .filter(account_id_sender.eq(id_sender.as_db_id()))
            .select((PendingMessageRaw::as_select(), message_bytes))
            .load(self.conn())
            .into_db_error(())
    }

    pub fn data_export_pending_messages(
        &mut self,
        id_sender: AccountIdInternal,
    ) -> Result<Vec<DataExportPendingMessage>, DieselDatabaseError> {
        let data = self
            .data_export_pending_messages_internal(id_sender)?
            .into_iter()
            .map(|(raw, message_data)| DataExportPendingMessage::new(raw, message_data))
            .collect();

        Ok(data)
    }

    pub fn admin_data_export_pending_messages(
        &mut self,
        id_sender: AccountIdInternal,
    ) -> Result<Vec<AdminDataExportPendingMessage>, DieselDatabaseError> {
        let data = self
            .data_export_pending_messages_internal(id_sender)?
            .into_iter()
            .map(|(raw, message_data)| AdminDataExportPendingMessage::new(raw, message_data))
            .collect();

        Ok(data)
    }
}
