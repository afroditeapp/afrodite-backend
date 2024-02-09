use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, AccountInteractionInternal};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_write_commands!(CurrentWriteChatInteraction, CurrentSyncWriteChatInteraction);

impl<C: ConnectionProvider> CurrentSyncWriteChatInteraction<C> {
    pub fn insert_account_interaction(
        &mut self,
        account1: AccountIdInternal,
        account2: AccountIdInternal,
    ) -> Result<AccountInteractionInternal, DieselDatabaseError> {
        use model::schema::{account_interaction::dsl::*, account_interaction_index::dsl::*};

        let interaction_value = insert_into(account_interaction)
            .default_values()
            .returning(AccountInteractionInternal::as_returning())
            .get_result::<AccountInteractionInternal>(self.conn())
            .into_db_error(DieselDatabaseError::Execute, (account1, account2))?;

        insert_into(account_interaction_index)
            .values((
                account_id_first.eq(account1.as_db_id()),
                account_id_second.eq(account2.as_db_id()),
                interaction_id.eq(interaction_value.id),
            ))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, (account1, account2))?;

        insert_into(account_interaction_index)
            .values((
                account_id_first.eq(account2.as_db_id()),
                account_id_second.eq(account1.as_db_id()),
                interaction_id.eq(interaction_value.id),
            ))
            .execute(self.conn())
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
        let interaction = self
            .read()
            .chat()
            .interaction()
            .account_interaction(account1, account2)?;

        match interaction {
            Some(interaction) => Ok(interaction),
            None => self.insert_account_interaction(account1, account2)
        }
    }
}
