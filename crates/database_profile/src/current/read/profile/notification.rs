use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, ProfileTextModerationCompletedNotification};
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

    pub fn profile_text_moderation_completed(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<ProfileTextModerationCompletedNotification, DieselDatabaseError> {
        use crate::schema::profile_app_notification_state::dsl::*;

        let query_result = profile_app_notification_state
            .filter(account_id.eq(account_id_value.as_db_id()))
            .select((
                profile_text_accepted,
                profile_text_accepted_viewed,
                profile_text_rejected,
                profile_text_rejected_viewed,
            ))
            .first::<(i64, i64, i64, i64)>(self.conn())
            .optional()
            .into_db_error(())?
            .map(|v| ProfileTextModerationCompletedNotification {
                accepted: v.0 as i8,
                accepted_viewed: v.1 as i8,
                rejected: v.2 as i8,
                rejected_viewed: v.3 as i8,
            });

        Ok(query_result.unwrap_or_default())
    }
}
