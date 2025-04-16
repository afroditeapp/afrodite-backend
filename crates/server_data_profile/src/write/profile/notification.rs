use database_profile::current::write::GetDbWriteCommandsProfile;
use model_profile::{AccountIdInternal, ProfileAppNotificationSettings, ProfileTextModerationCompletedNotificationViewed};
use server_data::{
    cache::CacheWriteCommon, define_cmd_wrapper_write, result::Result, write::DbTransaction, DataError, IntoDataError
};

define_cmd_wrapper_write!(WriteCommandsProfileNotification);

impl WriteCommandsProfileNotification<'_> {
    pub async fn upsert_app_notification_settings(
        &self,
        id: AccountIdInternal,
        value: ProfileAppNotificationSettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile().notification().upsert_app_notification_settings(id, value)
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
}
