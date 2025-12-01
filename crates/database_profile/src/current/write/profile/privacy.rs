use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::AccountIdInternal;
use model_profile::ProfilePrivacySettings;
use simple_backend_utils::db::MyRunQueryDsl;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfilePrivacy);

impl CurrentWriteProfilePrivacy<'_> {
    pub fn upsert_privacy_settings(
        &mut self,
        id: AccountIdInternal,
        settings: ProfilePrivacySettings,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_privacy_settings::dsl::*;

        insert_into(profile_privacy_settings)
            .values((account_id.eq(id.as_db_id()), settings))
            .on_conflict(account_id)
            .do_update()
            .set(settings)
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
