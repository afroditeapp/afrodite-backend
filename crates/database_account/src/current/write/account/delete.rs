use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{delete, prelude::*, update};
use model::{AccountIdInternal, UnixTime};
use simple_backend_utils::Result;

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

    fn set_account_deleted_unix_time_to_used_account_ids(
        &mut self,
        id_value: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::used_account_ids::dsl::*;

        update(used_account_ids.find(id_value.as_db_id()))
            .set(account_deleted_unix_time.eq(UnixTime::current_time()))
            .execute(self.conn())
            .into_db_error(id_value)?;

        Ok(())
    }

    pub fn delete_account(
        &mut self,
        id_value: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::account_id::dsl::*;

        self.set_account_deleted_unix_time_to_used_account_ids(id_value)?;

        delete(account_id)
            .filter(id.eq(id_value.as_db_id()))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn delete_old_initial_setup_used_account_ids(
        &mut self,
        deletion_unix_time_max: UnixTime,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::used_account_ids::dsl::*;

        delete(used_account_ids)
            .filter(
                account_deleted_unix_time
                    .lt(deletion_unix_time_max)
                    .and(initial_setup_completed.eq(false)),
            )
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
