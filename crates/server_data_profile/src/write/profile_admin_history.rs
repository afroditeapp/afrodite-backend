use model_profile::GetProfileStatisticsResult;
use server_data::{define_cmd_wrapper, result::Result, DataError};

use super::DbTransactionProfileHistory;

define_cmd_wrapper!(WriteCommandsProfileAdminHistory);

impl<C: DbTransactionProfileHistory> WriteCommandsProfileAdminHistory<C> {
    pub async fn save_profile_statistics(
        self,
        r: GetProfileStatisticsResult,
    ) -> Result<(), DataError> {
        db_transaction_history!(self, move |mut cmds| {
            cmds.profile_admin().statistics().save_statistics(r)
        })
    }
}
