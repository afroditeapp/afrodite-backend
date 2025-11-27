use std::collections::HashMap;

use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountId, AccountIdDb, AdminDataExportPendingMessage, ConversationId,
    DataExportPendingMessage, MessageId, MessageUuid, NewMessageNotification,
    NewMessageNotificationList, PendingMessageDbId, PendingMessageDbIdAndMessageTime,
    PendingMessageInfo, PendingMessageRaw, UnixTime,
};
use model_chat::{
    AccountIdInternal, DeliveryInfoType, GetSentMessage, MessageDeliveryInfo,
    PendingMessageInternal,
};

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

    pub fn new_message_notification_list(
        &mut self,
        id_message_receiver: AccountIdInternal,
    ) -> Result<(NewMessageNotificationList, Vec<PendingMessageDbId>), DieselDatabaseError> {
        use crate::schema::{account_id, account_interaction, pending_messages::dsl::*};

        let data: Vec<(
            i64,
            AccountIdDb,
            AccountId,
            AccountIdDb,
            ConversationId,
            ConversationId,
            bool,
        )> = pending_messages
            .inner_join(account_id::table.on(account_id_sender.eq(account_id::id)))
            .inner_join(account_interaction::table)
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .filter(receiver_acknowledgement.eq(false))
            .filter(account_interaction::account_id_sender.is_not_null())
            .filter(account_interaction::conversation_id_sender.is_not_null())
            .filter(account_interaction::conversation_id_receiver.is_not_null())
            .select((
                id,
                account_id::id,
                account_id::uuid,
                account_interaction::account_id_sender.assume_not_null(),
                account_interaction::conversation_id_sender.assume_not_null(),
                account_interaction::conversation_id_receiver.assume_not_null(),
                receiver_push_notification_sent,
            ))
            .order_by(account_id_sender)
            .load(self.conn())
            .into_db_error(())?;

        let mut notifications = HashMap::<AccountIdDb, (NewMessageNotification, bool)>::new();
        let mut messages_pending_push_notification = vec![];

        for (
            primary_key,
            sender_db_id,
            sender,
            like_sender_db_id,
            conversation_id_sender,
            conversation_id_receiver,
            push_notification_sent,
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
                messages_pending_push_notification.push(PendingMessageDbId { id: primary_key });
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
    ) -> Result<Vec<PendingMessageDbIdAndMessageTime>, DieselDatabaseError> {
        use crate::schema::pending_messages::dsl::*;

        let data: Vec<(i64, UnixTime)> = pending_messages
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .filter(receiver_acknowledgement.eq(false))
            .filter(receiver_email_notification_sent.eq(false))
            .select((id, message_unix_time))
            .order_by(account_id_sender)
            .load(self.conn())
            .into_db_error(())?;

        let v = data
            .into_iter()
            .map(|(primary_key, time)| PendingMessageDbIdAndMessageTime {
                id: primary_key,
                time,
            })
            .collect();

        Ok(v)
    }

    pub fn all_sent_messages(
        &mut self,
        id_message_sender: AccountIdInternal,
    ) -> Result<Vec<MessageUuid>, DieselDatabaseError> {
        use crate::schema::pending_messages::dsl::*;

        let value: Vec<PendingMessageInternal> = pending_messages
            .filter(account_id_sender.eq(id_message_sender.as_db_id()))
            .filter(sender_acknowledgement.eq(false))
            .select(PendingMessageInternal::as_select())
            .load(self.conn())
            .into_db_error(())?;

        let messages = value.into_iter().map(|msg| msg.message_uuid).collect();

        Ok(messages)
    }

    pub fn get_sent_message(
        &mut self,
        id_message_sender: AccountIdInternal,
        message: MessageUuid,
    ) -> Result<GetSentMessage, DieselDatabaseError> {
        use crate::schema::pending_messages::dsl::*;

        let value: Vec<u8> = pending_messages
            .filter(account_id_sender.eq(id_message_sender.as_db_id()))
            .filter(sender_acknowledgement.eq(false))
            .filter(message_uuid.eq(&message))
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

    pub fn has_unreceived_delivery_info(
        &mut self,
        sender_id: AccountIdInternal,
    ) -> Result<bool, DieselDatabaseError> {
        use crate::schema::message_delivery_info::dsl::*;

        let count: i64 = message_delivery_info
            .filter(account_id_sender.eq(sender_id.as_db_id()))
            .count()
            .get_result(self.conn())
            .into_db_error(())?;

        Ok(count > 0)
    }

    pub fn get_all_delivery_info(
        &mut self,
        sender_id: AccountIdInternal,
    ) -> Result<Vec<MessageDeliveryInfo>, DieselDatabaseError> {
        use crate::schema::{account_id, message_delivery_info};

        let data: Vec<(i64, AccountId, MessageId, DeliveryInfoType, UnixTime)> =
            message_delivery_info::table
                .inner_join(
                    account_id::table
                        .on(message_delivery_info::account_id_receiver.eq(account_id::id)),
                )
                .filter(message_delivery_info::account_id_sender.eq(sender_id.as_db_id()))
                .select((
                    message_delivery_info::id,
                    account_id::uuid,
                    message_delivery_info::message_id,
                    message_delivery_info::delivery_info_type,
                    message_delivery_info::unix_time,
                ))
                .load(self.conn())
                .into_db_error(())?;

        let result = data
            .into_iter()
            .map(
                |(id, receiver, message_id, delivery_type, unix_time)| MessageDeliveryInfo {
                    id,
                    receiver,
                    message_id,
                    delivery_type,
                    unix_time,
                },
            )
            .collect();

        Ok(result)
    }

    pub fn get_latest_seen_message_id(
        &mut self,
        viewer_id: AccountIdInternal,
        sender_id: AccountIdInternal,
    ) -> Result<Option<MessageId>, DieselDatabaseError> {
        use crate::schema::latest_seen_message::dsl::*;

        let result: Option<MessageId> = latest_seen_message
            .filter(account_id_viewer.eq(viewer_id.as_db_id()))
            .filter(account_id_sender.eq(sender_id.as_db_id()))
            .select(message_id)
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(result)
    }

    pub fn check_pending_message_info(
        &mut self,
        sender: AccountIdInternal,
        receiver: AccountIdInternal,
        message_uuid_value: MessageUuid,
    ) -> Result<Option<PendingMessageInfo>, DieselDatabaseError> {
        use crate::schema::pending_messages::dsl::*;

        let result: Option<(i64, MessageId)> = pending_messages
            .filter(account_id_sender.eq(sender.as_db_id()))
            .filter(account_id_receiver.eq(receiver.as_db_id()))
            .filter(message_uuid.eq(message_uuid_value))
            .select((id, message_id))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(
            result.map(|(private_key, message_id_value)| PendingMessageInfo {
                id: private_key,
                sender,
                m: message_id_value,
            }),
        )
    }
}
