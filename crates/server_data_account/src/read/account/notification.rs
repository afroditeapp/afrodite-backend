use model_account::{AccountAppNotificationSettings, AccountIdInternal};
use server_data::{cache::CacheReadCommon, define_cmd_wrapper_read, result::Result, DataError, IntoDataError};

define_cmd_wrapper_read!(ReadCommandsAccountNotification);

impl ReadCommandsAccountNotification<'_> {
    pub async fn account_app_notification_settings(
        &self,
        id: AccountIdInternal,
    ) -> Result<AccountAppNotificationSettings, DataError> {
        self.read_cache_common(id, |entry| Ok(entry.app_notification_settings.account))
            .await
            .into_error()
    }
}
