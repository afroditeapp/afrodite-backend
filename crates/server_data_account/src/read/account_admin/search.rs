use database_account::current::read::GetDbReadCommandsAccount;
use model_account::{EmailAddress, GetAccountIdFromEmailResult};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsAccountSearchAdmin);

impl ReadCommandsAccountSearchAdmin<'_> {
    pub async fn account_id_from_email(
        &self,
        email: EmailAddress,
    ) -> Result<GetAccountIdFromEmailResult, DataError> {
        self.db_read(move |mut cmds| {
            let value = cmds.account_admin().search().account_id_from_email(email)?;
            Ok(value)
        })
        .await
        .into_error()
    }
}
