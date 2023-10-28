use diesel::{insert_into, prelude::*, update, delete};
use error_stack::{Result, ResultExt};
use model::{
    AccessToken, Account, AccountId, AccountIdDb, AccountIdInternal, AccountSetup, RefreshToken,
    SignInWithInfo, AccountInteractionInternal, schema::{account_interaction_index, account_interaction::account_id_sender}, PendingMessageId, SendMessageToAccount, AccountInteractionState,
};
use utils::current_unix_time;

use crate::{diesel::{ConnectionProvider, DieselDatabaseError}, IntoDatabaseError, current::read::SqliteReadCommands, TransactionError};

use super::CurrentSyncWriteCommands;


define_write_commands!(CurrentWriteChat, CurrentSyncWriteChat);

impl<C: ConnectionProvider> CurrentSyncWriteChat<C> {
    pub fn insert_account_interaction(
        mut transaction_conn: C,
        account1: AccountIdInternal,
        account2: AccountIdInternal,
    ) -> Result<AccountInteractionInternal, DieselDatabaseError> {
        use model::schema::account_interaction_index::dsl::*;
        use model::schema::account_interaction::dsl::*;

        let interaction_value = insert_into(account_interaction)
            .default_values()
            .returning(AccountInteractionInternal::as_returning())
            .get_result::<AccountInteractionInternal>(transaction_conn.conn())
            .into_db_error(DieselDatabaseError::Execute, (account1, account2))?;

        insert_into(account_interaction_index)
            .values((
                account_id_first.eq(account1.as_db_id()),
                account_id_second.eq(account2.as_db_id()),
                interaction_id.eq(interaction_value.id)
            ))
            .execute(transaction_conn.conn())
            .into_db_error(DieselDatabaseError::Execute, (account1, account2))?;

        insert_into(account_interaction_index)
            .values((
                account_id_first.eq(account2.as_db_id()),
                account_id_second.eq(account1.as_db_id()),
                interaction_id.eq(interaction_value.id)
            ))
            .execute(transaction_conn.conn())
            .into_db_error(DieselDatabaseError::Execute, (account1, account2))?;

        Ok(interaction_value)
    }

    pub fn update_account_interaction(
        &mut self,
        value: AccountInteractionInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_interaction::dsl::*;

        let id_value = value.id;
        let account1 = value.account_id_sender;
        let account2 = value.account_id_receiver;

        update(account_interaction.find(value.id))
            .set(value)
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, (id_value, account1, account2))?;

        Ok(())
    }

    pub fn get_or_create_account_interaction(
        &mut self,
        account1: AccountIdInternal,
        account2: AccountIdInternal,
    ) -> Result<AccountInteractionInternal, DieselDatabaseError> {
        let value = self.conn().transaction(|mut conn| {
            let interaction = conn.read().chat().account_interaction(account1, account2)?;
            match interaction {
                Some(interaction) => Ok(interaction),
                None => {
                    let value = CurrentSyncWriteChat::insert_account_interaction(conn, account1, account2)?;
                    Ok::<_, TransactionError<_>>(value)
                }
            }
        })?;

        Ok(value)
    }

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
                        message_number.eq(message.message_number)
                            .and(account_id_receiver.eq(message_receiver.as_db_id()))
                        )
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
        use model::schema::pending_messages::dsl::*;
        use model::schema::account_interaction;
        let time = current_unix_time();
        let interaction = self.get_or_create_account_interaction(sender, receiver)?;
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
                .into_db_error(DieselDatabaseError::Execute, (sender, receiver, new_message_number))?;

            insert_into(pending_messages)
                .values((
                    account_id_sender.eq(sender.as_db_id()),
                    account_id_receiver.eq(receiver.as_db_id()),
                    unix_time.eq(time),
                    message_number.eq(new_message_number),
                    message_text.eq(message),
                ))
                .execute(conn)
                .into_db_error(DieselDatabaseError::Execute, (sender, receiver, new_message_number))?;

            Ok::<_, TransactionError<_>>(())
        })?;

        Ok(())
    }

}
