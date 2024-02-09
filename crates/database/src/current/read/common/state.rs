use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, AccountState, Capabilities, SharedState, SharedStateInternal};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_read_commands!(CurrentReadAccountState, CurrentSyncReadCommonState);

impl<C: ConnectionProvider> CurrentSyncReadCommonState<C> {
    pub fn shared_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<SharedState, DieselDatabaseError> {
        use crate::schema::shared_state::dsl::*;

        let data: SharedStateInternal = shared_state
            .filter(account_id.eq(id.as_db_id()))
            .select(SharedStateInternal::as_select())
            .first(self.conn())
            .into_db_error(id)?;

        let state: AccountState = TryInto::<AccountState>::try_into(data.account_state_number)
            .change_context(DieselDatabaseError::DataFormatConversion)?;

        Ok(SharedState {
            account_state: state,
            is_profile_public: data.is_profile_public,
        })
    }

    pub fn account_capabilities(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Capabilities, DieselDatabaseError> {
        use crate::schema::account_capabilities::dsl::*;

        account_capabilities
            .filter(account_id.eq(id.as_db_id()))
            .select(Capabilities::as_select())
            .first(self.conn())
            .into_db_error(id)
    }
}
