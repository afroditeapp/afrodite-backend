use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use model::{AccountIdInternal, LoginSessionInfo};
use model_account::AccountLockedState;
use simple_backend_utils::Result;

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

    pub fn login_session_info(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<LoginSessionInfo>, DieselDatabaseError> {
        use crate::schema::login_session_info::dsl::*;

        login_session_info
            .filter(account_id.eq(id.as_db_id()))
            .select(LoginSessionInfo::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(id)
    }
}
