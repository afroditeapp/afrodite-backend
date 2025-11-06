use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{prelude::*, update};
use error_stack::Result;
use model::AccountIdInternal;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountLockAdmin);

impl CurrentWriteAccountLockAdmin<'_> {
    pub fn set_locked_state(
        &mut self,
        id: AccountIdInternal,
        locked: bool,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::account_state::dsl::*;

        update(account_state)
            .filter(account_id.eq(id.as_db_id()))
            .set(account_locked.eq(locked))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
