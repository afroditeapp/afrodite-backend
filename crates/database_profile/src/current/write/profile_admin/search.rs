use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::AccountIdInternal;
use model_profile::AutomaticProfileSearchLastSeenUnixTime;
use simple_backend_utils::db::MyRunQueryDsl;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileAdminSearch);

impl CurrentWriteProfileAdminSearch<'_> {
    pub fn upsert_automatic_profile_search_last_seen_time(
        &mut self,
        id: AccountIdInternal,
        last_seen_time: AutomaticProfileSearchLastSeenUnixTime,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_automatic_profile_search_state::dsl::*;

        insert_into(profile_automatic_profile_search_state)
            .values((
                account_id.eq(id.as_db_id()),
                last_seen_unix_time.eq(last_seen_time),
            ))
            .on_conflict(account_id)
            .do_update()
            .set(last_seen_unix_time.eq(last_seen_time))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
