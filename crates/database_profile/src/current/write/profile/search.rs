use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{ExpressionMethods, insert_into, prelude::*};
use error_stack::Result;
use model::AccountIdInternal;
use model_profile::AutomaticProfileSearchSettings;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileSearch);

impl CurrentWriteProfileSearch<'_> {
    pub fn upsert_automatic_profile_search_settings(
        &mut self,
        id: AccountIdInternal,
        settings: AutomaticProfileSearchSettings,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_automatic_profile_search_settings::dsl::*;

        insert_into(profile_automatic_profile_search_settings)
            .values((account_id.eq(id.as_db_id()), settings))
            .on_conflict(account_id)
            .do_update()
            .set(settings)
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
