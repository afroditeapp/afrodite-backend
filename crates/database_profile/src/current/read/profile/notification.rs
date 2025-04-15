use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::AccountIdInternal;
use model_profile::ProfileAppNotificationSettings;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadProfileNotification);

impl CurrentReadProfileNotification<'_> {
    pub fn app_notification_settings(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<ProfileAppNotificationSettings, DieselDatabaseError> {
        use crate::schema::profile_app_notification_settings::dsl::*;

        let query_result = profile_app_notification_settings
            .filter(account_id.eq(account_id_value.as_db_id()))
            .select(ProfileAppNotificationSettings::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(query_result.unwrap_or_default())
    }
}
