use database_account::current::read::GetDbReadCommandsAccount;
use model_account::{AccountAppNotificationSettings, AccountIdInternal};
use server_data::{define_cmd_wrapper_read, read::DbRead, result::Result, DataError};

define_cmd_wrapper_read!(ReadCommandsAccountNotification);

impl ReadCommandsAccountNotification<'_> {
    pub async fn account_app_notification_settings(
        &self,
        id: AccountIdInternal,
    ) -> Result<AccountAppNotificationSettings, DataError> {
        let state = self
            .db_read(move |mut cmds| cmds.account().notification().app_notification_settings(id))
            .await?;
        Ok(state)
    }
}
