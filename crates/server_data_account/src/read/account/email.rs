use database_account::current::read::GetDbReadCommandsAccount;
use model::{AccountIdInternal, EmailLoginTokenRow, UnixTime};
use model_account::{
    AccountEmailSendingStateRaw, EmailChangeLimits, EmailLoginLimits, EmailVerificationLimits,
};
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

    pub async fn email_login_limits(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<EmailLoginLimits>, DataError> {
        self.db_read(move |mut cmds| cmds.account().email().email_login_limits(id))
            .await
            .into_error()
    }

    pub async fn email_change_limits(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<EmailChangeLimits>, DataError> {
        self.db_read(move |mut cmds| cmds.account().email().email_change_limits(id))
            .await
            .into_error()
    }

    pub async fn email_verification_limits(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<EmailVerificationLimits>, DataError> {
        self.db_read(move |mut cmds| cmds.account().email().email_verification_limits(id))
            .await
            .into_error()
    }

    pub async fn email_registration_limits(
        &self,
    ) -> Result<Option<model::EmailRegistrationLimits>, DataError> {
        self.db_read(move |mut cmds| cmds.account().email().email_registration_limits())
            .await
            .into_error()
    }

    pub async fn email_address_history_entries(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<model_account::EmailAddressHistoryEntry>, DataError> {
        self.db_read(move |mut cmds| cmds.account().email().email_address_history_entries(id))
            .await
            .into_error()
    }

    pub async fn email_address_history_count(
        &self,
        id: AccountIdInternal,
    ) -> Result<i64, DataError> {
        self.db_read(move |mut cmds| cmds.account().email().email_address_history_count(id))
            .await
            .into_error()
    }

    pub async fn email_address_history_oldest_change_unix_time(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<UnixTime>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account()
                .email()
                .email_address_history_oldest_change_unix_time(id)
        })
        .await
        .into_error()
    }
}
