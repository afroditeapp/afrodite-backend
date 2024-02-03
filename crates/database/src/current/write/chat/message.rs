use diesel::{delete, insert_into, prelude::*, update};
use error_stack::Result;
use model::{
    AccountIdInternal, AccountInteractionInternal, AccountInteractionState, PendingMessageId,
};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};
use simple_backend_utils::current_unix_time;

use crate::{current::read::CurrentSyncReadCommands, IntoDatabaseError, TransactionError};

define_write_commands!(CurrentWriteChatMessage, CurrentSyncWriteChatMessage);

impl<C: ConnectionProvider> CurrentSyncWriteChatMessage<C> {

    pub fn delete_pending_message_list(
        &mut self,
        message_receiver: AccountIdInternal,
        messages: Vec<PendingMessageId>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::pending_messages::dsl::*;

        self.conn().transaction(|mut conn| {
            for message in messages {
                delete(
                    pending_messages.filter(
                        message_number
                            .eq(message.message_number)
                            .and(account_id_receiver.eq(message_receiver.as_db_id())),
                    ),
                )
                .execute(conn.conn())
                .into_db_error(DieselDatabaseError::Execute, message_receiver)?;
            }
            Ok::<_, TransactionError<_>>(())
        })?;

        Ok(())
    }

    pub fn insert_pending_message_if_match(
        &mut self,
        sender: AccountIdInternal,
        receiver: AccountIdInternal,
        message: String,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::{account_interaction, pending_messages::dsl::*};
        let time = current_unix_time();
        let interaction = self.cmds().chat().interaction().get_or_create_account_interaction(sender, receiver)?;
        // Skip message number 0, so that latest viewed message number
        // does not have that message already viewed.
        let new_message_number = interaction.message_counter + 1;

        if interaction.state_number != AccountInteractionState::Match as i64 {
            return Err(DieselDatabaseError::NotAllowed.into());
        }

        self.conn().transaction(|conn| {
            update(account_interaction::table.find(interaction.id))
                .set(account_interaction::message_counter.eq(new_message_number))
                .execute(conn)
                .into_db_error(
                    DieselDatabaseError::Execute,
                    (sender, receiver, new_message_number),
                )?;

            insert_into(pending_messages)
                .values((
                    account_id_sender.eq(sender.as_db_id()),
                    account_id_receiver.eq(receiver.as_db_id()),
                    unix_time.eq(time),
                    message_number.eq(new_message_number),
                    message_text.eq(message),
                ))
                .execute(conn)
                .into_db_error(
                    DieselDatabaseError::Execute,
                    (sender, receiver, new_message_number),
                )?;

            Ok::<_, TransactionError<_>>(())
        })?;

        Ok(())
    }
}
