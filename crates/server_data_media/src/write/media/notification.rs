use database_media::current::write::GetDbWriteCommandsMedia;
use model::{AccountIdInternal, MediaContentModerationCompletedNotificationViewed};
use model_media::MediaAppNotificationSettings;
use server_data::{
    DataError, IntoDataError, cache::CacheWriteCommon, db_transaction, define_cmd_wrapper_write,
    result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsMediaNotification);

impl WriteCommandsMediaNotification<'_> {
    pub async fn upsert_app_notification_settings(
        &self,
        id: AccountIdInternal,
        value: MediaAppNotificationSettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .notification()
                .upsert_app_notification_settings(id, value)
        })?;

        self.write_cache_common(id, |entry| {
            entry.app_notification_settings.media = value;
            Ok(())
        })
        .await
        .into_error()
    }

    pub async fn update_notification_viewed_values(
        &self,
        id: AccountIdInternal,
        values: MediaContentModerationCompletedNotificationViewed,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .notification()
                .update_notification_viewed_values(id, values)?;
            Ok(())
        })?;

        Ok(())
    }
}
