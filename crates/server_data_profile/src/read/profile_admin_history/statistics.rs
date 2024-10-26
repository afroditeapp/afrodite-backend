use model::{GetProfileStatisticsHistoryResult, ProfileStatisticsHistoryValueTypeInternal};
use server_data::{
    define_server_data_read_commands, read::ReadCommandsProvider, result::Result, DataError, IntoDataError,
};

define_server_data_read_commands!(ReadCommandsProfileAdminHistoryStatistics);
define_db_read_history_command!(ReadCommandsProfileAdminHistoryStatistics);

impl<C: ReadCommandsProvider> ReadCommandsProfileAdminHistoryStatistics<C> {
    pub async fn profile_statistics(
        &mut self,
        settings: ProfileStatisticsHistoryValueTypeInternal,
    ) -> Result<GetProfileStatisticsHistoryResult, DataError> {

        self
            .db_read_history(move |mut cmds| {
                cmds.profile_admin().statistics().profile_statistics_history(settings)
            })
            .await
            .into_data_error(())
    }
}
