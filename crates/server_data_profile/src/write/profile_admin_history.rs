use database_profile::history::write::GetDbHistoryWriteCommandsProfile;
use model_profile::ProfileStatisticsInternal;
use server_data::{
    define_cmd_wrapper_write, result::Result, write::DbTransactionHistory, DataError,
};

define_cmd_wrapper_write!(WriteCommandsProfileAdminHistory);

impl WriteCommandsProfileAdminHistory<'_> {
    pub async fn save_profile_statistics(
        &self,
        r: ProfileStatisticsInternal,
    ) -> Result<(), DataError> {
        db_transaction_history!(self, move |mut cmds| {
            cmds.profile_admin_history().statistics().save_statistics(r)
        })
    }
}
