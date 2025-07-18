use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::AccountIdInternal;
use model_profile::AutomaticProfileSearchLastSeenUnixTime;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadProfileAdminSearch);

impl CurrentReadProfileAdminSearch<'_> {
    pub fn automatic_profile_search_last_seen_time(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<Option<AutomaticProfileSearchLastSeenUnixTime>, DieselDatabaseError> {
        use crate::schema::profile_automatic_profile_search_state::dsl::*;

        let query_result = profile_automatic_profile_search_state
            .filter(account_id.eq(account_id_value.as_db_id()))
            .select(last_seen_unix_time)
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(query_result)
    }
}
