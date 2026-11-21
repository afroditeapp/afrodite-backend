use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use model::{AccountIdInternal, UnixTime};
use simple_backend_utils::Result;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountDelete);

impl CurrentReadAccountDelete<'_> {
    pub fn account_deletion_requested(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<UnixTime>, DieselDatabaseError> {
        use crate::schema::account_state::dsl::*;

        account_state
            .filter(account_id.eq(id.as_db_id()))
            .select(account_deletion_request_unix_time)
            .first(self.conn())
            .into_db_error(id)
    }

    /// Returns whether the deletion request was initiated automatically
    /// e.g. for inactive account.
    pub fn account_deletion_requested_is_automatic(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<bool, DieselDatabaseError> {
        use crate::schema::account_state::dsl::*;

        account_state
            .filter(account_id.eq(id.as_db_id()))
            .select(account_deletion_request_automatic)
            .first(self.conn())
            .into_db_error(id)
    }
}
