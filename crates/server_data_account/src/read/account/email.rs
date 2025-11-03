use database_account::current::read::GetDbReadCommandsAccount;
use model_account::{AccountEmailSendingStateRaw, AccountIdInternal};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsAccountEmail);

impl ReadCommandsAccountEmail<'_> {
    pub async fn email_state(
        &self,
        id: AccountIdInternal,
    ) -> Result<AccountEmailSendingStateRaw, DataError> {
        let state = self
            .db_read(move |mut cmds| cmds.account().email().email_sending_states(id))
            .await?;
        Ok(state)
    }

    pub async fn account_id_from_email(
        &self,
        email: model_account::EmailAddress,
    ) -> Result<Option<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| cmds.account().email().account_id_from_email(email))
            .await
            .into_error()
    }
}
