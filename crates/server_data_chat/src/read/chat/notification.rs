use model::AccountIdInternal;
use model_chat::ChatAppNotificationSettings;
use server_data::{cache::CacheReadCommon, define_cmd_wrapper_read, result::Result, DataError, IntoDataError};

define_cmd_wrapper_read!(ReadCommandsChatNotification);

impl ReadCommandsChatNotification<'_> {
    pub async fn chat_app_notification_settings(
        &self,
        id: AccountIdInternal,
    ) -> Result<ChatAppNotificationSettings, DataError> {
        self.read_cache_common(id, |entry| Ok(entry.app_notification_settings.chat))
            .await
            .into_error()
    }
}
