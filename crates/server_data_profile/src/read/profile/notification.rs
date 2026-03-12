use model_profile::{
    AccountIdInternal, AutomaticProfileSearchCompletedNotification, ProfileAppNotificationSettings,
};
use server_data::{
    DataError, IntoDataError, cache::CacheReadCommon, define_cmd_wrapper_read, result::Result,
};

use crate::cache::CacheReadProfile;

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

    pub async fn automatic_profile_search_completed(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<AutomaticProfileSearchCompletedNotification, DataError> {
        self.read_cache_profile_and_common(account_id, |p, _| {
            Ok(p.automatic_profile_search.notification)
        })
        .await
        .into_error()
    }
}
