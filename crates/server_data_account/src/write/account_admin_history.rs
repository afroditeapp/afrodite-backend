use std::collections::HashMap;

use database_account::history::write::GetDbHistoryWriteCommandsAccount;
use model::ClientVersion;
use server_data::{
    DataError, define_cmd_wrapper_write, result::Result, write::DbTransactionHistory,
};

define_cmd_wrapper_write!(WriteCommandsAccountAdminHistory);

impl WriteCommandsAccountAdminHistory<'_> {
    pub async fn save_client_version_statistics(
        &self,
        statistics: HashMap<ClientVersion, i64>,
    ) -> Result<(), DataError> {
        db_transaction_history!(self, move |mut cmds| {
            cmds.account_admin_history()
                .client_version()
                .save_client_version_statistics(statistics)
        })
    }
}
