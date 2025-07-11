use database_profile::current::write::GetDbWriteCommandsProfile;
use model_profile::{
    AccountIdInternal, AutomaticProfileSearchCompletedNotificationViewed,
    ProfileAppNotificationSettings, ProfileTextModerationCompletedNotificationViewed,
};
use server_data::{
    DataError, IntoDataError, cache::CacheWriteCommon, db_transaction, define_cmd_wrapper_write,
    result::Result, write::DbTransaction,
};

use crate::cache::CacheWriteProfile;

define_cmd_wrapper_write!(WriteCommandsProfileNotification);

impl WriteCommandsProfileNotification<'_> {
    pub async fn upsert_app_notification_settings(
        &self,
        id: AccountIdInternal,
        value: ProfileAppNotificationSettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .notification()
                .upsert_app_notification_settings(id, value)
        })?;

        self.write_cache_common(id, |entry| {
            entry.app_notification_settings.profile = value;
            Ok(())
        })
        .await
        .into_error()
    }

    pub async fn update_notification_viewed_values(
        &self,
        id: AccountIdInternal,
        values: ProfileTextModerationCompletedNotificationViewed,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .notification()
                .update_notification_viewed_values(id, values)?;
            Ok(())
        })?;

        Ok(())
    }

    pub async fn update_automatic_profile_search_notification_viewed_values(
        &self,
        id: AccountIdInternal,
        values: AutomaticProfileSearchCompletedNotificationViewed,
    ) -> Result<(), DataError> {
        self.write_cache_profile(id, |p| {
            p.automatic_profile_search
                .notification
                .profiles_found_viewed = values.profiles_found;
            Ok(())
        })
        .await
        .into_error()
    }
}
