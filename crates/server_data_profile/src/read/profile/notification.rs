use database_profile::current::read::GetDbReadCommandsProfile;
use model_profile::{
    AccountIdInternal, AutomaticProfileSearchCompletedNotification, ProfileAppNotificationSettings,
    ProfileTextModerationCompletedNotification,
};
use server_data::{
    DataError, IntoDataError, cache::CacheReadCommon, define_cmd_wrapper_read, read::DbRead,
    result::Result,
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

    pub async fn profile_text_moderation_completed(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<ProfileTextModerationCompletedNotification, DataError> {
        let info = self
            .db_read(move |mut cmds| {
                cmds.profile()
                    .notification()
                    .profile_text_moderation_completed(account_id)
            })
            .await
            .into_error()?;

        Ok(info)
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
