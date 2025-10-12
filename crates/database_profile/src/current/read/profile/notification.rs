use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::{
    AccountIdInternal, NotificationId, NotificationIdViewed, NotificationStatus,
    ProfileStringModerationCompletedNotification,
};
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

    pub fn profile_string_moderation_completed(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<ProfileStringModerationCompletedNotification, DieselDatabaseError> {
        use crate::schema::profile_app_notification_state::dsl::*;

        let query_result = profile_app_notification_state
            .filter(account_id.eq(account_id_value.as_db_id()))
            .select((
                profile_name_accepted,
                profile_name_accepted_viewed,
                profile_name_rejected,
                profile_name_rejected_viewed,
                profile_text_accepted,
                profile_text_accepted_viewed,
                profile_text_rejected,
                profile_text_rejected_viewed,
            ))
            .first::<(
                NotificationId,
                NotificationIdViewed,
                NotificationId,
                NotificationIdViewed,
                NotificationId,
                NotificationIdViewed,
                NotificationId,
                NotificationIdViewed,
            )>(self.conn())
            .optional()
            .into_db_error(())?
            .map(|v| ProfileStringModerationCompletedNotification {
                name_accepted: NotificationStatus {
                    id: v.0,
                    viewed: v.1,
                },
                name_rejected: NotificationStatus {
                    id: v.2,
                    viewed: v.3,
                },
                text_accepted: NotificationStatus {
                    id: v.4,
                    viewed: v.5,
                },
                text_rejected: NotificationStatus {
                    id: v.6,
                    viewed: v.7,
                },
                hidden: false,
            });

        Ok(query_result.unwrap_or_default())
    }
}
