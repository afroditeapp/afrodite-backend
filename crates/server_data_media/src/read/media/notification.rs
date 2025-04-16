use database_media::current::read::GetDbReadCommandsMedia;
use model::{AccountIdInternal, MediaContentModerationCompletedNotification};
use model_media::MediaAppNotificationSettings;
use server_data::{cache::CacheReadCommon, define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError};

define_cmd_wrapper_read!(ReadCommandsMediaNotification);

impl ReadCommandsMediaNotification<'_> {
    pub async fn chat_app_notification_settings(
        &self,
        id: AccountIdInternal,
    ) -> Result<MediaAppNotificationSettings, DataError> {
        self.read_cache_common(id, |entry| Ok(entry.app_notification_settings.media))
            .await
            .into_error()
    }

    pub async fn media_content_moderation_completed(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<MediaContentModerationCompletedNotification, DataError> {
        let info = self.db_read(move |mut cmds| {
            cmds.media()
                .notification()
                .media_content_moderation_completed(account_id)
        })
        .await
        .into_error()?;

        Ok(info)
    }
}
