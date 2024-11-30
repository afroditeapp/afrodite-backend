use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, AccountStateRelatedSharedState, OtherSharedState, Permissions};
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::{define_current_read_commands, IntoDatabaseError};

define_current_read_commands!(CurrentReadCommonState);

impl CurrentReadCommonState<'_> {
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

    pub fn other_shared_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<OtherSharedState, DieselDatabaseError> {
        use crate::schema::shared_state::dsl::*;

        shared_state
            .filter(account_id.eq(id.as_db_id()))
            .select(OtherSharedState::as_select())
            .first(self.conn())
            .into_db_error(id)
    }

    pub fn account_permissions(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Permissions, DieselDatabaseError> {
        use crate::schema::account_permissions::dsl::*;

        account_permissions
            .filter(account_id.eq(id.as_db_id()))
            .select(Permissions::as_select())
            .first(self.conn())
            .into_db_error(id)
    }
}
