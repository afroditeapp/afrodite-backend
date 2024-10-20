use database::{define_current_write_commands, ConnectionProvider, DieselDatabaseError};
use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{
    AccountGlobalState, AccountIdInternal, AccountInternal, ClientId, EmailAddress, SetAccountSetup
};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountData, CurrentSyncWriteAccountData);

impl<C: ConnectionProvider> CurrentSyncWriteAccountData<C> {
    pub fn insert_account(
        &mut self,
        id: AccountIdInternal,
        account_data: AccountInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        insert_into(account)
            .values((account_id.eq(id.as_db_id()), email.eq(account_data.email)))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn account(
        mut self,
        id: AccountIdInternal,
        account_data: &AccountInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        update(account.find(id.as_db_id()))
            .set(account_data)
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn insert_default_account_setup(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_setup::dsl::*;

        insert_into(account_setup)
            .values(account_id.eq(id.as_db_id()))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn account_setup(
        &mut self,
        id: AccountIdInternal,
        data: &SetAccountSetup,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_setup::dsl::*;

        if let Some(birthdate_value) = &data.birthdate {
            self.common_write_access().common().state().update_birthdate(id, *birthdate_value)?;
        }

        update(account_setup.find(id.as_db_id()))
            .set((
                birthdate.eq(data.birthdate),
                is_adult.eq(Some(data.is_adult)),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn insert_account_state(
        mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        insert_into(account_state)
            .values((
                account_id.eq(id.as_db_id()),
            ))
            .execute(self.conn())
            .into_db_error(())?;
        Ok(())
    }

    pub fn upsert_increment_admin_access_granted_count(
        &mut self,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_global_state::dsl::*;

        insert_into(account_global_state)
            .values((
                row_type.eq(AccountGlobalState::ACCOUNT_GLOBAL_STATE_ROW_TYPE),
                admin_access_granted_count.eq(1),
            ))
            .on_conflict(row_type)
            .do_update()
            .set(admin_access_granted_count.eq(admin_access_granted_count + 1))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn update_account_email(
        mut self,
        id: AccountIdInternal,
        email_address: &EmailAddress,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        update(account.find(id.as_db_id()))
            .set(email.eq(email_address))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn get_next_client_id(
        mut self,
        id: AccountIdInternal,
    ) -> Result<ClientId, DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        let current: ClientId = account_state
            .filter(account_id.eq(id.as_db_id()))
            .select(next_client_id)
            .first(self.conn())
            .optional()
            .into_db_error(())?
            .unwrap_or_default();

        let next = current.increment();

        insert_into(account_state)
            .values((
                account_id.eq(id.as_db_id()),
                next_client_id.eq(next),
            ))
            .on_conflict(account_id)
            .do_update()
            .set(next_client_id.eq(next))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(current)
    }
}
