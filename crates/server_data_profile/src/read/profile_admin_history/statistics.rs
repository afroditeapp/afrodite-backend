use model_profile::{GetProfileStatisticsHistoryResult, ProfileStatisticsHistoryValueTypeInternal};
use server_data::{
    define_cmd_wrapper, result::Result, DataError, IntoDataError
};

use crate::read::DbReadProfileHistory;

define_cmd_wrapper!(ReadCommandsProfileAdminHistoryStatistics);

impl<C: DbReadProfileHistory> ReadCommandsProfileAdminHistoryStatistics<C> {
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
