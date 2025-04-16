use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::MediaContentModerationCompletedNotification;
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

    pub fn media_content_moderation_completed(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<MediaContentModerationCompletedNotification, DieselDatabaseError> {
        use crate::schema::media_app_notification_state::dsl::*;

        let query_result = media_app_notification_state
            .filter(account_id.eq(account_id_value.as_db_id()))
            .select((
                media_content_accepted,
                media_content_accepted_viewed,
                media_content_rejected,
                media_content_rejected_viewed,
            ))
            .first::<(i64, i64, i64, i64)>(self.conn())
            .optional()
            .into_db_error(())?
            .map(|v| MediaContentModerationCompletedNotification {
                accepted: v.0 as i8,
                accepted_viewed: v.1 as i8,
                rejected: v.2 as i8,
                rejected_viewed: v.3 as i8,
            });

        Ok(query_result.unwrap_or_default())
    }
}
