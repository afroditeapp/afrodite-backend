use database_chat::current::write::GetDbWriteCommandsChat;
use model::AccountIdInternal;
use model_chat::ChatAppNotificationSettings;
use server_data::{
    cache::CacheWriteCommon, define_cmd_wrapper_write, result::Result, write::DbTransaction, DataError, IntoDataError
};

define_cmd_wrapper_write!(WriteCommandsChatNotification);

impl WriteCommandsChatNotification<'_> {
    pub async fn upsert_app_notification_settings(
        &self,
        id: AccountIdInternal,
        value: ChatAppNotificationSettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().notification().upsert_app_notification_settings(id, value)
        })?;

        self.write_cache_common(id, |entry| {
            entry.app_notification_settings.chat = value;
            Ok(())
        })
            .await
            .into_error()
    }
}
