use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*, ExpressionMethods};
use error_stack::Result;
use model::{AccountIdInternal, ProfileTextModerationCompletedNotificationViewed};
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

    pub fn update_notification_viewed_values(
        &mut self,
        id: AccountIdInternal,
        values: ProfileTextModerationCompletedNotificationViewed,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_app_notification_state::dsl::*;

        insert_into(profile_app_notification_state)
            .values((
                account_id.eq(id.as_db_id()),
            ))
            .on_conflict(account_id)
            .do_update()
            .set((
                profile_text_accepted_viewed.eq(Into::<i64>::into(values.accepted)),
                profile_text_rejected_viewed.eq(Into::<i64>::into(values.rejected)),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
