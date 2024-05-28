use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountData, AccountGlobalState, AccountId, AccountIdInternal, AccountInternal, AccountSetup, ACCOUNT_GLOBAL_STATE_ROW_TYPE
};
use simple_backend_database::
    diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_read_commands!(CurrentReadAccountData, CurrentSyncReadAccountData);

impl<C: ConnectionProvider> CurrentSyncReadAccountData<C> {
    pub fn account_ids_internal(
        &mut self,
    ) -> Result<Vec<AccountIdInternal>, DieselDatabaseError> {
        use crate::schema::account_id::dsl::*;

        account_id
            .select(AccountIdInternal::as_select())
            .load(self.conn())
            .into_db_error(())
    }

    pub fn account_ids(
        &mut self,
    ) -> Result<Vec<AccountId>, DieselDatabaseError> {
        use crate::schema::account_id::dsl::*;

        account_id
            .select(uuid)
            .load(self.conn())
            .into_db_error(())
    }

    pub fn account_setup(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AccountSetup, DieselDatabaseError> {
        use crate::schema::account_setup::dsl::*;

        account_setup
            .filter(account_id.eq(id.as_db_id()))
            .select(AccountSetup::as_select())
            .first(self.conn())
            .into_db_error(id)
    }

    pub fn account_data(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AccountData, DieselDatabaseError> {
        use crate::schema::account::dsl::*;

        let account_internal = account
            .filter(account_id.eq(id.as_db_id()))
            .select(AccountInternal::as_select())
            .first(self.conn())
            .into_db_error(id)?;

        Ok(AccountData {
            email: account_internal.email,
        })
    }

    pub fn global_state(
        &mut self,
    ) -> Result<AccountGlobalState, DieselDatabaseError> {
        use model::schema::account_global_state::dsl::*;

        account_global_state
            .filter(row_type.eq(ACCOUNT_GLOBAL_STATE_ROW_TYPE))
            .select(AccountGlobalState::as_select())
            .first(self.conn())
            .optional()
            .map(|v| v.unwrap_or_default())
            .into_db_error(())
    }
}
