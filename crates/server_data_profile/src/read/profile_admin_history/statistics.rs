use database_profile::history::read::GetDbReadCommandsProfileHistory;
use model_profile::{GetProfileStatisticsHistoryResult, ProfileStatisticsHistoryValueTypeInternal};
use server_data::{DataError, IntoDataError, define_cmd_wrapper_read, result::Result};

use crate::read::DbReadProfileHistory;

define_cmd_wrapper_read!(ReadCommandsProfileAdminHistoryStatistics);

impl ReadCommandsProfileAdminHistoryStatistics<'_> {
    pub async fn profile_statistics(
        &self,
        settings: ProfileStatisticsHistoryValueTypeInternal,
    ) -> Result<GetProfileStatisticsHistoryResult, DataError> {
        self.db_read_history(move |mut cmds| {
            cmds.profile_admin()
                .statistics()
                .profile_statistics_history(settings)
        })
        .await
        .into_data_error(())
    }
}
