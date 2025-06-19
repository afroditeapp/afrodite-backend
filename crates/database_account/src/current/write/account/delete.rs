use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{delete, prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, UnixTime};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountDelete);

impl CurrentWriteAccountDelete<'_> {
    pub fn set_account_deletion_request_state(
        &mut self,
        id: AccountIdInternal,
        value: bool,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_state::dsl::*;

        let state = if value {
            Some(UnixTime::current_time())
        } else {
            None
        };

        update(account_state)
            .filter(account_id.eq(id.as_db_id()))
            .set(account_deletion_request_unix_time.eq(state))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn delete_account(
        &mut self,
        id_value: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_id::dsl::*;

        delete(account_id)
            .filter(id.eq(id_value.as_db_id()))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
