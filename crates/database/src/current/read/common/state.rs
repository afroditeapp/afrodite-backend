use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, AccountStateRelatedSharedState, Capabilities};
use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};

use crate::IntoDatabaseError;

define_read_commands!(CurrentReadAccountState, CurrentSyncReadCommonState);

impl<C: ConnectionProvider> CurrentSyncReadCommonState<C> {
    pub fn account_state_related_shared_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AccountStateRelatedSharedState, DieselDatabaseError> {
        use crate::schema::shared_state::dsl::*;

        shared_state
            .filter(account_id.eq(id.as_db_id()))
            .select(AccountStateRelatedSharedState::as_select())
            .first(self.conn())
            .into_db_error(id)
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
