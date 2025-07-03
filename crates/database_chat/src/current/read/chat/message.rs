use std::collections::HashMap;

use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountId, AccountIdDb, ConversationId, NewMessageNotification, NewMessageNotificationList,
};
use model_chat::{
    AccountIdInternal, GetSentMessage, PendingMessageDbId, PendingMessageInternal, SentMessageId,
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
    ) -> Result<(NewMessageNotificationList, Vec<PendingMessageDbId>), DieselDatabaseError> {
        use crate::schema::{account_id, account_interaction, pending_messages::dsl::*};

        let data: Vec<(
            PendingMessageDbId,
            AccountId,
            AccountIdDb,
            ConversationId,
            ConversationId,
            bool,
        )> = pending_messages
            .inner_join(
                account_id::table.on(account_id_sender.assume_not_null().eq(account_id::id)),
            )
            .inner_join(account_interaction::table)
            .filter(account_id_receiver.eq(id_message_receiver.as_db_id()))
            .filter(receiver_acknowledgement.eq(false))
            .filter(account_interaction::account_id_sender.is_not_null())
            .filter(account_interaction::conversation_id_sender.is_not_null())
            .filter(account_interaction::conversation_id_receiver.is_not_null())
            .select((
                id,
                account_id::uuid,
                account_interaction::account_id_sender.assume_not_null(),
                account_interaction::conversation_id_sender.assume_not_null(),
                account_interaction::conversation_id_receiver.assume_not_null(),
                receiver_push_notification_sent,
            ))
            .order_by(account_id::id)
            .load(self.conn())
            .into_db_error(())?;

        let mut notifications = HashMap::<AccountId, (NewMessageNotification, bool)>::new();
        let mut messages_pending_push_notification = vec![];

        for (
            pending_message_id,
            a,
            like_sender,
            conversation_id_sender,
            conversation_id_receiver,
            push_notification_sent,
        ) in data
        {
            // Select message receiver specific conversation ID
            let c = if like_sender == id_message_receiver.into_db_id() {
                conversation_id_sender
            } else {
                conversation_id_receiver
            };
            let mut entry = notifications
                .entry(a)
                .insert_entry((NewMessageNotification { a, c, m: 0 }, true));
            entry.get_mut().0.m += 1;

            if !push_notification_sent {
                // Message notification needs an update
                entry.get_mut().1 = false;
                messages_pending_push_notification.push(pending_message_id);
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
}
