use database_account::current::read::GetDbReadCommandsAccount;
use model::AccountIdInternal;
use model_account::AccountLockedState;
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsAccountLockAdmin);

impl ReadCommandsAccountLockAdmin<'_> {
    pub async fn account_locked_state(
        &self,
        id: AccountIdInternal,
    ) -> Result<AccountLockedState, DataError> {
        self.db_read(move |mut cmds| cmds.account_admin().login().account_locked_state(id))
            .await
            .into_error()
    }
}
