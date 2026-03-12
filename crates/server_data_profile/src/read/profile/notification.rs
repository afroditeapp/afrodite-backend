use model_profile::{AccountIdInternal, ProfileAppNotificationSettings};
use server_data::{
    DataError, IntoDataError, cache::CacheReadCommon, define_cmd_wrapper_read, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsProfileNotification);

impl ReadCommandsProfileNotification<'_> {
    pub async fn chat_app_notification_settings(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileAppNotificationSettings, DataError> {
        self.read_cache_common(id, |entry| Ok(entry.app_notification_settings.profile))
            .await
            .into_error()
    }
}
