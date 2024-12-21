use database::{define_current_read_commands, DieselDatabaseError};
use diesel::{prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, UnixTime};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentWriteAccountBanAdmin);

impl CurrentWriteAccountBanAdmin<'_> {
    pub fn set_banned_until_time(
        &mut self,
        id: AccountIdInternal,
        banned_until: Option<UnixTime>,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::account_state::dsl::*;

        update(account_state)
            .filter(account_id.eq(id.as_db_id()))
            .set(account_banned_until_unix_time.eq(banned_until))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
