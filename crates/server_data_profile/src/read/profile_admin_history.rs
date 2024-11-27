use server_data::define_cmd_wrapper;

use super::DbReadProfileHistory;

mod statistics;

define_cmd_wrapper!(ReadCommandsProfileAdminHistory);

impl<C: DbReadProfileHistory> ReadCommandsProfileAdminHistory<C> {
    pub fn statistics(self) -> statistics::ReadCommandsProfileAdminHistoryStatistics<C> {
        statistics::ReadCommandsProfileAdminHistoryStatistics::new(self.0)
    }
}
