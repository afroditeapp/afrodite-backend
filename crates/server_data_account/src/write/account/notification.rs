use database_account::current::write::GetDbWriteCommandsAccount;
use model_account::{AccountAppNotificationSettings, AccountIdInternal};
use server_data::{
    DataError, IntoDataError, cache::CacheWriteCommon, db_transaction, define_cmd_wrapper_write,
    result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsAccountNotification);

impl WriteCommandsAccountNotification<'_> {
    pub async fn upsert_app_notification_settings(
        &self,
        id: AccountIdInternal,
        value: AccountAppNotificationSettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .notification()
                .upsert_app_notification_settings(id, value)
        })?;

        self.write_cache_common(id, |entry| {
            entry.app_notification_settings.account = value;
            Ok(())
        })
        .await
        .into_error()
    }
}
