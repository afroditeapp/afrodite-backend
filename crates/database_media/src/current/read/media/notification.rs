use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model_media::{AccountIdInternal, MediaAppNotificationSettings};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadMediaNotification);

impl CurrentReadMediaNotification<'_> {
    pub fn app_notification_settings(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<MediaAppNotificationSettings, DieselDatabaseError> {
        use crate::schema::media_app_notification_settings::dsl::*;

        let query_result = media_app_notification_settings
            .filter(account_id.eq(account_id_value.as_db_id()))
            .select(MediaAppNotificationSettings::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(query_result.unwrap_or_default())
    }
}
