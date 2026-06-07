use database_account::current::read::GetDbReadCommandsAccount;
use model_account::CustomEmail;
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
}
