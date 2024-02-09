use diesel::{insert_into, prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, AccountState, Capabilities, SharedState, SharedStateInternal};
use simple_backend_database::diesel_db::DieselDatabaseError;

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_write_commands!(CurrentWriteAccountState, CurrentSyncWriteCommonState);

impl<C: ConnectionProvider> CurrentSyncWriteCommonState<C> {
    pub fn insert_shared_state(
        &mut self,
        id: AccountIdInternal,
        data: SharedStateInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        insert_into(shared_state)
            .values((
                account_id.eq(id.as_db_id()),
                is_profile_public.eq(&data.is_profile_public),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn shared_state(
        &mut self,
        id: AccountIdInternal,
        data: SharedState,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        update(shared_state.find(id.as_db_id()))
            .set((
                account_state_number.eq(data.account_state as i64),
                is_profile_public.eq(&data.is_profile_public),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn insert_default_account_capabilities(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_capabilities::dsl::*;

        insert_into(account_capabilities)
            .values((account_id.eq(id.as_db_id()),))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn account_capabilities(
        &mut self,
        id: AccountIdInternal,
        data: Capabilities,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_capabilities::dsl::*;

        update(account_capabilities.find(id.as_db_id()))
            .set(data)
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn account_state(
        mut self,
        id: AccountIdInternal,
        state: AccountState,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::shared_state::dsl::*;

        update(shared_state.find(id.as_db_id()))
            .set(account_state_number.eq(state as i64))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }
}
