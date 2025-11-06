use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::AccountIdInternal;
use model_account::AccountLockedState;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountLock);

impl CurrentReadAccountLock<'_> {
    pub fn account_locked_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<AccountLockedState, DieselDatabaseError> {
        use crate::schema::account_state::dsl::*;

        account_state
            .filter(account_id.eq(id.as_db_id()))
            .select(account_locked)
            .first(self.conn())
            .into_db_error(id)
            .map(|locked| AccountLockedState { locked })
    }
}
