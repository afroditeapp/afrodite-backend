use database_account::current::read::GetDbReadCommandsAccount;
use model::AccountIdInternal;
use model_account::AccountReport;
use server_data::{define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError};

define_cmd_wrapper_read!(ReadCommandsAccountReport);

impl ReadCommandsAccountReport<'_> {
    pub async fn get_report(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
    ) -> Result<AccountReport, DataError> {
        self.db_read(move |mut cmds| cmds.account().report().get_report(creator, target))
            .await
            .into_error()
    }
}
