use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{AccountId, AccountIdDb, AccountIdInternal, AccountInternal, AccountSetup};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_write_commands!(CurrentWriteAccountData, CurrentSyncWriteAccountData);

impl<C: ConnectionProvider> CurrentSyncWriteAccountData<C> {
    pub fn insert_account_id(
        mut self,
        account_uuid: AccountId,
    ) -> Result<AccountIdInternal, DieselDatabaseError> {
        use model::schema::account_id::dsl::*;

        let db_id: AccountIdDb = insert_into(account_id)
            .values(uuid.eq(account_uuid))
            .returning(id)
            .get_result(self.conn())
            .into_db_error(DieselDatabaseError::Execute, account_uuid)?;

        Ok(AccountIdInternal {
            uuid: account_uuid,
            id: db_id,
        })
    }

    pub fn insert_account(
        &mut self,
        id: AccountIdInternal,
        account_data: AccountInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account::dsl::*;

        insert_into(account)
            .values((account_id.eq(id.as_db_id()), email.eq(account_data.email)))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

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
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn insert_account_setup(
        &mut self,
        id: AccountIdInternal,
        account_data: &AccountSetup,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_setup::dsl::*;

        insert_into(account_setup)
            .values((account_id.eq(id.as_db_id()), account_data))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn account_setup(
        &mut self,
        id: AccountIdInternal,
        account_data: &AccountSetup,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_setup::dsl::*;

        update(account_setup.find(id.as_db_id()))
            .set(account_data)
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }
}
