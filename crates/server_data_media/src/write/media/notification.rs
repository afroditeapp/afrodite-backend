use database_media::current::write::GetDbWriteCommandsMedia;
use model::AccountIdInternal;
use model_media::MediaAppNotificationSettings;
use server_data::{
    cache::CacheWriteCommon, define_cmd_wrapper_write, result::Result, write::DbTransaction, DataError, IntoDataError
};

define_cmd_wrapper_write!(WriteCommandsMediaNotification);

impl WriteCommandsMediaNotification<'_> {
    pub async fn upsert_app_notification_settings(
        &self,
        id: AccountIdInternal,
        value: MediaAppNotificationSettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media().notification().upsert_app_notification_settings(id, value)
        })?;

        self.write_cache_common(id, |entry| {
            entry.app_notification_settings.media = value;
            Ok(())
        })
            .await
            .into_error()
    }
}
