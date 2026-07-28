use database_account::current::read::GetDbReadCommandsAccount;
use model::{AccountIdInternal, EmailLoginTokenRow, UnixTime};
use model_account::AccountEmailSendingStateRaw;
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

    /// Load all login tokens from DB at once (for startup restoration).
    pub async fn all_email_login_tokens(&self) -> Result<Vec<EmailLoginTokenRow>, DataError> {
        self.db_read(move |mut cmds| cmds.account().email().all_email_login_tokens())
            .await
            .into_error()
    }

    pub async fn email_login_token_sent_time(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<UnixTime>, DataError> {
        self.db_read(move |mut cmds| cmds.account().email().email_login_token_sent_time(id))
            .await
            .into_error()
    }
}
