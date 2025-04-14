use database_account::current::write::GetDbWriteCommandsAccount;
use model_account::{AccountAppNotificationSettings, AccountIdInternal};
use server_data::{
    define_cmd_wrapper_write, result::Result, write::DbTransaction, DataError,
};

define_cmd_wrapper_write!(WriteCommandsAccountNotification);

impl WriteCommandsAccountNotification<'_> {
    pub async fn upsert_app_notification_settings(
        &self,
        id: AccountIdInternal,
        value: AccountAppNotificationSettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().notification().upsert_app_notification_settings(id, value)
        })
    }
}
