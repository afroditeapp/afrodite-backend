use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, ExpressionMethods};
use error_stack::Result;
use model::AccountIdInternal;
use model_profile::ProfileAppNotificationSettings;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileNotification);

impl CurrentWriteProfileNotification<'_> {
    pub fn upsert_app_notification_settings(
        &mut self,
        id: AccountIdInternal,
        settings: ProfileAppNotificationSettings,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_app_notification_settings::dsl::*;

        insert_into(profile_app_notification_settings)
            .values((
                account_id.eq(id.as_db_id()),
                settings,
            ))
            .on_conflict(account_id)
            .do_update()
            .set(settings)
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
