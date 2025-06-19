use database_account::current::read::GetDbReadCommandsAccount;
use model_account::{AccountIdInternal, GetAccountBanTimeResult};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsAccountBan);

impl ReadCommandsAccountBan<'_> {
    pub async fn ban_time(
        &self,
        id: AccountIdInternal,
    ) -> Result<GetAccountBanTimeResult, DataError> {
        self.db_read(move |mut cmds| cmds.account().ban().account_ban_time(id))
            .await
            .into_error()
    }
}
