use database_account::current::write::GetDbWriteCommandsAccount;
use model::AccountIdInternal;
use model_account::{AppleAccountId, GoogleAccountId};
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
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
}
