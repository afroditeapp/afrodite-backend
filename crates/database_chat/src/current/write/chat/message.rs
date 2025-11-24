use std::sync::Arc;

use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{delete, insert_into, prelude::*, update};
use error_stack::{Result, ResultExt};
use model::PublicKeyId;
use model_chat::{
    AccountIdInternal, AccountInteractionState, ClientId, ClientLocalId, DeliveryInfoType,
    PendingMessageIdInternal, SentMessageId, SignedMessageData, UnixTime,
};
use utils::encrypt::ParsedKeys;

use super::ReceiverBlockedSender;
use crate::{IntoDatabaseError, current::write::GetDbWriteCommandsChat};

define_current_write_commands!(CurrentWriteChatMessage);

impl CurrentWriteChatMessage<'_> {
    pub fn mark_receiver_push_notification_sent(
        &mut self,
        messages: Vec<PendingMessageIdInternal>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_messages::dsl::*;

        for m in messages {
            update(pending_messages)
                .filter(account_id_sender.eq(m.sender.as_db_id()))
                .filter(account_id_receiver.eq(m.receiver))
                .filter(message_id.eq(m.m))
                .set(receiver_push_notification_sent.eq(true))
                .execute(self.conn())
                .into_db_error(())?;
        }

        Ok(())
    }

    pub fn mark_message_email_notification_sent(
        &mut self,
        messages: Vec<PendingMessageIdInternal>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_messages::dsl::*;

        for m in messages {
            update(pending_messages)
                .filter(account_id_sender.eq(m.sender.as_db_id()))
                .filter(account_id_receiver.eq(m.receiver))
                .filter(message_id.eq(m.m))
                .set(receiver_email_notification_sent.eq(true))
                .execute(self.conn())
                .into_db_error(())?;
        }

        Ok(())
    }

    pub fn add_receiver_acknowledgement_and_delete_if_also_sender_has_acknowledged(
        &mut self,
        message_receiver: AccountIdInternal,
        messages: Vec<PendingMessageIdInternal>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_messages::dsl::*;

        for message in messages {
            update(pending_messages)
                .filter(message_id.eq(message.m))
                .filter(account_id_sender.eq(message.sender.as_db_id()))
                .filter(account_id_receiver.eq(message_receiver.as_db_id()))
                .set(receiver_acknowledgement.eq(true))
                .execute(self.conn())
                .into_db_error(message_receiver)?;

            delete(pending_messages)
                .filter(message_id.eq(message.m))
                .filter(account_id_sender.eq(message.sender.as_db_id()))
                .filter(account_id_receiver.eq(message_receiver.as_db_id()))
                .filter(sender_acknowledgement.eq(true))
                .filter(receiver_acknowledgement.eq(true))
                .execute(self.conn())
                .into_db_error(message_receiver)?;
        }

        Ok(())
    }

    pub fn add_sender_acknowledgement_and_delete_if_also_receiver_has_acknowledged(
        &mut self,
        message_sender: AccountIdInternal,
        messages: Vec<SentMessageId>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_messages::dsl::*;

        for message in messages {
            update(pending_messages)
                .filter(sender_client_id.eq(message.c))
                .filter(sender_client_local_id.eq(message.l))
                .filter(account_id_sender.eq(message_sender.as_db_id()))
                .set(sender_acknowledgement.eq(true))
                .execute(self.conn())
                .into_db_error(message_sender)?;

            delete(pending_messages)
                .filter(sender_client_id.eq(message.c))
                .filter(sender_client_local_id.eq(message.l))
                .filter(account_id_sender.eq(message_sender.as_db_id()))
                .filter(sender_acknowledgement.eq(true))
                .filter(receiver_acknowledgement.eq(true))
                .execute(self.conn())
                .into_db_error(message_sender)?;
        }

        Ok(())
    }

    /// Returns PGP signed message containing [SignedMessageData]
    /// in binary format.
    #[allow(clippy::too_many_arguments)]
    pub fn insert_pending_message_if_match_and_not_blocked(
        &mut self,
        sender: AccountIdInternal,
        receiver: AccountIdInternal,
        sender_public_key_id: PublicKeyId,
        receiver_public_key_id: PublicKeyId,
        message: Vec<u8>,
        client_id_value: ClientId,
        client_local_id_value: ClientLocalId,
        keys: Arc<ParsedKeys>,
    ) -> Result<std::result::Result<Vec<u8>, ReceiverBlockedSender>, DieselDatabaseError> {
        use model::schema::{account_interaction, pending_messages::dsl::*};
        let time = UnixTime::current_time();
        let interaction = self
            .write()
            .chat()
            .interaction()
            .get_or_create_account_interaction(sender, receiver)?;

        if interaction.is_direction_blocked(receiver, sender) {
            return Ok(Err(ReceiverBlockedSender));
        }

        // The is_blocked handles the case where sender has blocked the
        // message receiver.
        if interaction.state_number != AccountInteractionState::Match || interaction.is_blocked() {
            return Err(DieselDatabaseError::NotAllowed.into());
        }

        let new_message_id = interaction.next_message_id();

        if interaction.account_id_sender == Some(*sender.as_db_id()) {
            update(account_interaction::table.find(interaction.id))
                .set(
                    account_interaction::message_counter_sender
                        .eq(account_interaction::message_counter_sender + 1),
                )
                .execute(self.conn())
                .into_db_error((sender, receiver, new_message_id))?;
        } else {
            update(account_interaction::table.find(interaction.id))
                .set(
                    account_interaction::message_counter_receiver
                        .eq(account_interaction::message_counter_receiver + 1),
                )
                .execute(self.conn())
                .into_db_error((sender, receiver, new_message_id))?;
        }

        let data_for_signing = SignedMessageData {
            sender: sender.as_id(),
            receiver: receiver.as_id(),
            sender_public_key_id,
            receiver_public_key_id,
            m: new_message_id,
            unix_time: time,
            message,
        };

        let signed = keys
            .sign(data_for_signing.to_bytes())
            .change_context(DieselDatabaseError::MessageEncryptionError)?;

        insert_into(pending_messages)
            .values((
                account_interaction.eq(interaction.id),
                account_id_sender.eq(sender.as_db_id()),
                account_id_receiver.eq(receiver.as_db_id()),
                message_unix_time.eq(time),
                message_id.eq(new_message_id),
                message_bytes.eq(&signed),
                sender_client_id.eq(client_id_value),
                sender_client_local_id.eq(client_local_id_value),
            ))
            .execute(self.conn())
            .into_db_error((sender, receiver, new_message_id))?;

        Ok(Ok(signed))
    }

    pub fn insert_message_delivery_info(
        &mut self,
        sender: AccountIdInternal,
        receiver: AccountIdInternal,
        message_id_value: model::MessageId,
        delivery_info_type_value: DeliveryInfoType,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::message_delivery_info::dsl::*;

        let time = UnixTime::current_time();

        insert_into(message_delivery_info)
            .values((
                account_id_sender.eq(sender.as_db_id()),
                account_id_receiver.eq(receiver.as_db_id()),
                message_id.eq(message_id_value),
                delivery_info_type.eq(delivery_info_type_value),
                unix_time.eq(time),
            ))
            .execute(self.conn())
            .into_db_error((sender, receiver))?;

        Ok(())
    }

    pub fn delete_delivery_info_by_ids(
        &mut self,
        sender_id: AccountIdInternal,
        ids_to_delete: Vec<i64>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::message_delivery_info::dsl::*;

        delete(message_delivery_info)
            .filter(account_id_sender.eq(sender_id.as_db_id()))
            .filter(id.eq_any(ids_to_delete))
            .execute(self.conn())
            .into_db_error(sender_id)?;

        Ok(())
    }
}
