use model_profile::GetProfileStatisticsResult;
use server_data::{define_server_data_write_commands, write::WriteCommandsProvider, DataError, result::Result};

define_server_data_write_commands!(WriteCommandsProfileAdminHistory);
define_db_transaction_history_command!(WriteCommandsProfileAdminHistory);

impl<C: WriteCommandsProvider> WriteCommandsProfileAdminHistory<C> {
    pub async fn save_profile_statistics(
        self,
        r: GetProfileStatisticsResult,
    ) -> Result<(), DataError> {
        db_transaction_history!(self, move |mut cmds| {
            cmds.profile_admin().statistics().save_statistics(r)
        })
    }
}
