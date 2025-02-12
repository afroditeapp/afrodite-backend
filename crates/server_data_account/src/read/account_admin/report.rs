use database_account::current::read::GetDbReadCommandsAccount;
use model_account::GetAccountReportList;
use server_data::{
    define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError,
};

define_cmd_wrapper_read!(ReadCommandsAccountReport);

impl ReadCommandsAccountReport<'_> {
    pub async fn get_report_list(
        &self,
    ) -> Result<GetAccountReportList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account_admin()
                .report()
                .get_report_list()
        })
        .await
        .into_error()
    }
}
