use std::{collections::HashMap, sync::Arc};

use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{delete, insert_into, prelude::*, update};
use error_stack::{Result, ResultExt};
use model::{PendingMessageInfo, PublicKeyId};
use model_chat::{
    AccountIdInternal, AccountInteractionState, DeliveryInfoType, MessageId, SignedMessageData,
    UnixTime,
};
use simple_backend_utils::db::MyRunQueryDsl;
use utils::encrypt::ParsedKeys;

use super::RecipientBlockedSender;
use crate::{IntoDatabaseError, current::write::GetDbWriteCommandsChat};

define_current_write_commands!(CurrentWriteChatMessage);

impl CurrentWriteChatMessage<'_> {
    pub fn add_recipient_acknowledgement_and_delete_if_also_sender_has_acknowledged(
        &mut self,
        message_recipient: AccountIdInternal,
        messages: &[PendingMessageInfo],
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_messages::dsl::*;

        for message in messages {
            update(pending_messages)
                .filter(id.eq(message.id))
                .set(recipient_acknowledgement.eq(true))
                .execute(self.conn())
                .into_db_error(message_recipient)?;

            delete(pending_messages)
                .filter(id.eq(message.id))
                .filter(sender_acknowledgement.eq(true))
                .filter(recipient_acknowledgement.eq(true))
                .execute(self.conn())
                .into_db_error(message_recipient)?;
        }

        Ok(())
    }

    pub fn add_sender_acknowledgement_and_delete_if_also_recipient_has_acknowledged(
        &mut self,
        message_sender: AccountIdInternal,
        messages: Vec<MessageId>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_messages::dsl::*;

        for message in messages {
            update(pending_messages)
                .filter(message_id.eq(&message))
                .filter(account_id_sender.eq(message_sender.as_db_id()))
                .set(sender_acknowledgement.eq(true))
                .execute(self.conn())
                .into_db_error(message_sender)?;

            delete(pending_messages)
                .filter(message_id.eq(&message))
                .filter(account_id_sender.eq(message_sender.as_db_id()))
                .filter(sender_acknowledgement.eq(true))
                .filter(recipient_acknowledgement.eq(true))
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
        recipient: AccountIdInternal,
        sender_public_key_id_value: PublicKeyId,
        recipient_public_key_id: PublicKeyId,
        message: Vec<u8>,
        message_id_value: MessageId,
        keys: Arc<ParsedKeys>,
    ) -> Result<std::result::Result<Vec<u8>, RecipientBlockedSender>, DieselDatabaseError> {
        use model::schema::{account_interaction, pending_messages::dsl::*};
        let time = UnixTime::current_time();
        let interaction = self
            .write()
            .chat()
            .interaction()
            .get_or_create_account_interaction(sender, recipient)?;

        if interaction.is_direction_blocked(recipient, sender) {
            return Ok(Err(RecipientBlockedSender));
        }

        // The is_blocked handles the case where sender has blocked the
        // message recipient.
        if interaction.state_number != AccountInteractionState::Match || interaction.is_blocked() {
            return Err(DieselDatabaseError::NotAllowed.into());
        }

        let new_message_number = interaction.next_message_number();

        if interaction.account_id_sender == Some(*sender.as_db_id()) {
            update(account_interaction::table.find(interaction.id))
                .set(
                    account_interaction::message_counter_sender
                        .eq(account_interaction::message_counter_sender + 1),
                )
                .execute(self.conn())
                .into_db_error((sender, recipient, new_message_number))?;
        } else {
            update(account_interaction::table.find(interaction.id))
                .set(
                    account_interaction::message_counter_recipient
                        .eq(account_interaction::message_counter_recipient + 1),
                )
                .execute(self.conn())
                .into_db_error((sender, recipient, new_message_number))?;
        }

        let data_for_signing = SignedMessageData {
            sender: sender.as_id(),
            recipient: recipient.as_id(),
            message_id: message_id_value,
            sender_public_key_id: sender_public_key_id_value,
            recipient_public_key_id,
            m: new_message_number,
            unix_time: time,
            message,
        };

        let signed = keys
            .sign(data_for_signing.to_bytes())
            .change_context(DieselDatabaseError::MessageEncryptionError)?;

        insert_into(pending_messages)
            .values((
                account_id_sender.eq(sender.as_db_id()),
                account_id_recipient.eq(recipient.as_db_id()),
                sender_public_key_id.eq(sender_public_key_id_value),
                message_unix_time.eq(time),
                message_number.eq(new_message_number),
                message_bytes.eq(&signed),
                message_id.eq(&message_id_value),
            ))
            .execute(self.conn())
            .into_db_error((sender, recipient, new_message_number))?;

        Ok(Ok(signed))
    }

    pub fn insert_message_delivery_info(
        &mut self,
        sender: AccountIdInternal,
        recipient: AccountIdInternal,
        message_id_value: model::MessageId,
        delivery_info_type_value: DeliveryInfoType,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::message_delivery_info::dsl::*;

        let time = UnixTime::current_time();

        insert_into(message_delivery_info)
            .values((
                account_id_sender.eq(sender.as_db_id()),
                account_id_recipient.eq(recipient.as_db_id()),
                message_id.eq(message_id_value),
                delivery_info_type.eq(delivery_info_type_value),
                unix_time.eq(time),
            ))
            .execute(self.conn())
            .into_db_error((sender, recipient))?;

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

    pub fn update_latest_seen_message(
        &mut self,
        viewer_id: AccountIdInternal,
        sender_id: AccountIdInternal,
        msg_number: model::MessageNumber,
    ) -> Result<(), DieselDatabaseError> {
        use diesel::upsert::excluded;

        {
            use model::schema::latest_seen_message::dsl::*;

            insert_into(latest_seen_message)
                .values((
                    account_id_viewer.eq(viewer_id.as_db_id()),
                    account_id_sender.eq(sender_id.as_db_id()),
                    message_number.eq(msg_number),
                ))
                .on_conflict((account_id_viewer, account_id_sender))
                .do_update()
                .set(message_number.eq(excluded(message_number)))
                .execute_my_conn(self.conn())
                .into_db_error((viewer_id, sender_id))?;
        }

        let current_time = model::UnixTime::current_time();

        {
            use model::schema::latest_seen_message_pending_delivery::dsl::*;

            insert_into(latest_seen_message_pending_delivery)
                .values((
                    account_id_viewer.eq(viewer_id.as_db_id()),
                    account_id_sender.eq(sender_id.as_db_id()),
                    message_number.eq(msg_number),
                    unix_time.eq(current_time),
                ))
                .on_conflict((account_id_viewer, account_id_sender))
                .do_update()
                .set((
                    message_number.eq(excluded(message_number)),
                    unix_time.eq(excluded(unix_time)),
                ))
                .execute_my_conn(self.conn())
                .into_db_error((viewer_id, sender_id))?;
        }

        Ok(())
    }

    pub fn delete_pending_latest_seen_message_deliveries(
        &mut self,
        sender_id: AccountIdInternal,
        acknowledged: HashMap<AccountIdInternal, model::MessageNumber>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::latest_seen_message_pending_delivery::dsl::*;

        for (viewer, acknowledged_message_number) in acknowledged {
            delete(latest_seen_message_pending_delivery)
                .filter(account_id_sender.eq(sender_id.as_db_id()))
                .filter(account_id_viewer.eq(viewer.as_db_id()))
                .filter(message_number.eq(acknowledged_message_number))
                .execute(self.conn())
                .into_db_error(sender_id)?;
        }

        Ok(())
    }

    pub fn upsert_conversation_id(
        &mut self,
        owner_id: AccountIdInternal,
        other_id: AccountIdInternal,
        conversation_id_value: model_chat::ConversationId,
    ) -> Result<(), DieselDatabaseError> {
        use diesel::upsert::excluded;
        use model::schema::conversation_id::dsl::*;

        insert_into(conversation_id)
            .values((
                account_id.eq(owner_id.as_db_id()),
                other_account_id.eq(other_id.as_db_id()),
                id.eq(conversation_id_value),
            ))
            .on_conflict((account_id, other_account_id))
            .do_update()
            .set(id.eq(excluded(id)))
            .execute_my_conn(self.conn())
            .into_db_error((owner_id, other_id))?;

        Ok(())
    }
}
