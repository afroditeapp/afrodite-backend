use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::{AccountIdInternal, MediaContentModerationCompletedNotificationViewed};
use model_media::MediaAppNotificationSettings;
use simple_backend_utils::db::MyRunQueryDsl;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteMediaNotification);

impl CurrentWriteMediaNotification<'_> {
    pub fn upsert_app_notification_settings(
        &mut self,
        id: AccountIdInternal,
        settings: MediaAppNotificationSettings,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_app_notification_settings::dsl::*;

        insert_into(media_app_notification_settings)
            .values((account_id.eq(id.as_db_id()), settings))
            .on_conflict(account_id)
            .do_update()
            .set(settings)
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn update_notification_viewed_values(
        &mut self,
        id: AccountIdInternal,
        values: MediaContentModerationCompletedNotificationViewed,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::media_app_notification_state::dsl::*;

        insert_into(media_app_notification_state)
            .values((
                account_id.eq(id.as_db_id()),
                media_content_accepted_viewed.eq(values.accepted),
                media_content_rejected_viewed.eq(values.rejected),
                media_content_deleted_viewed.eq(values.deleted),
            ))
            .on_conflict(account_id)
            .do_update()
            .set((
                media_content_accepted_viewed.eq(values.accepted),
                media_content_rejected_viewed.eq(values.rejected),
                media_content_deleted_viewed.eq(values.deleted),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
