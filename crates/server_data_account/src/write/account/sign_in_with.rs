use database_account::current::write::GetDbWriteCommandsAccount;
use model::{AccountIdInternal, UnixTime};
use model_account::{AppleAccountId, GoogleAccountId};
use server_data::{
    DataError, app::GetConfig, db_transaction, define_cmd_wrapper_write, result::Result,
    write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsAccountSignInWith);

impl WriteCommandsAccountSignInWith<'_> {
    pub async fn update_apple_account_id(
        &self,
        id: AccountIdInternal,
        apple_id: Option<AppleAccountId>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .sign_in_with()
                .update_apple_account_id(id, apple_id)
        })
    }

    pub async fn update_google_account_id(
        &self,
        id: AccountIdInternal,
        google_id: Option<GoogleAccountId>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .sign_in_with()
                .update_google_account_id(id, google_id)
        })
    }

    pub async fn prune_sign_in_with_history(&self) -> Result<(), DataError> {
        let retention_duration = self
            .config()
            .limits_account()
            .sign_in_with_history_retention_duration;

        let retention_unix_time = UnixTime::new(
            UnixTime::current_time().ut - Into::<i64>::into(retention_duration.seconds),
        );

        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .sign_in_with()
                .prune_sign_in_with_history(retention_unix_time)
        })
    }
}
