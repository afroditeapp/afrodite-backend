use database_profile::history::write::GetDbHistoryWriteCommandsProfile;
use model_profile::GetProfileStatisticsResult;
use server_data::{define_cmd_wrapper_write, result::Result, DataError, write::DbTransactionHistory};

define_cmd_wrapper_write!(WriteCommandsProfileAdminHistory);

impl WriteCommandsProfileAdminHistory<'_> {
    pub async fn save_profile_statistics(
        &self,
        r: GetProfileStatisticsResult,
    ) -> Result<(), DataError> {
        db_transaction_history!(self, move |mut cmds| {
            cmds.profile_admin_history().statistics().save_statistics(r)
        })
    }
}
