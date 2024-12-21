use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, UnixTime};

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
}
