use model::AccountIdInternal;
use model_media::MediaAppNotificationSettings;
use server_data::{cache::CacheReadCommon, define_cmd_wrapper_read, result::Result, DataError, IntoDataError};

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
}
