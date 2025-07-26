use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::{
    MediaContentModerationCompletedNotification, NotificationId, NotificationIdViewed,
    NotificationStatus,
};
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
                media_content_deleted,
                media_content_deleted_viewed,
            ))
            .first::<(
                NotificationId,
                NotificationIdViewed,
                NotificationId,
                NotificationIdViewed,
                NotificationId,
                NotificationIdViewed,
            )>(self.conn())
            .optional()
            .into_db_error(())?
            .map(|v| MediaContentModerationCompletedNotification {
                accepted: NotificationStatus {
                    id: v.0,
                    viewed: v.1,
                },
                rejected: NotificationStatus {
                    id: v.2,
                    viewed: v.3,
                },
                deleted: NotificationStatus {
                    id: v.4,
                    viewed: v.5,
                },
            });

        Ok(query_result.unwrap_or_default())
    }
}
