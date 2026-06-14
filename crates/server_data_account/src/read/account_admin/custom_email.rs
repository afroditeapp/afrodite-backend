use database_account::current::read::GetDbReadCommandsAccount;
use model::{AccountIdInternal, CustomEmailSendingLimits};
use model_account::{CustomEmail, CustomEmailId, CustomEmailTranslation};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsAccountCustomEmailAdmin);

impl ReadCommandsAccountCustomEmailAdmin<'_> {
    pub async fn custom_email_list_page(&self, page: u32) -> Result<Vec<CustomEmail>, DataError> {
        self.db_read(move |mut cmds| {
            let value = cmds
                .account_admin()
                .custom_email()
                .custom_email_list_page(page.into())?;
            Ok(value)
        })
        .await
        .into_error()
    }

    pub async fn custom_email_unsent_accounts(
        &self,
        email_id_value: CustomEmailId,
    ) -> Result<Vec<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .custom_email_unsent_accounts(email_id_value)
        })
        .await
        .into_error()
    }

    pub async fn custom_emails_pending_sending(&self) -> Result<Vec<CustomEmailId>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .custom_emails_pending_sending()
        })
        .await
        .into_error()
    }

    pub async fn custom_email_translations(
        &self,
        email_id_value: CustomEmailId,
    ) -> Result<Vec<CustomEmailTranslation>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .custom_email_translations(email_id_value)
        })
        .await
        .into_error()
    }

    pub async fn custom_email_sending_limits(
        &self,
    ) -> Result<Option<CustomEmailSendingLimits>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .custom_email_sending_limits()
        })
        .await
        .into_error()
    }
}
