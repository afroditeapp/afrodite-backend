use database_account::history::read::GetDbHistoryReadCommandsAccount;
use model_account::{GetClientVersionStatisticsResult, GetClientVersionStatisticsSettings};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsAccountAdminHistory);

impl ReadCommandsAccountAdminHistory<'_> {
    pub async fn get_client_version_statistics(
        &self,
        settings: GetClientVersionStatisticsSettings,
    ) -> Result<GetClientVersionStatisticsResult, DataError> {
        self.db_read_history(move |mut cmds| {
            cmds.account_admin_history()
                .client_version()
                .client_version_statistics(settings)
        })
        .await
        .into_error()
    }
}
