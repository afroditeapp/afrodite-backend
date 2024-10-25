use database::{define_history_write_commands, ConnectionProvider};

mod statistics;

define_history_write_commands!(HistoryWriteProfileAdmin, HistorySyncWriteProfileAdmin);

impl<C: ConnectionProvider> HistorySyncWriteProfileAdmin<C> {
    pub fn statistics(self) -> statistics::HistorySyncWriteProfileAdminStatistics<C> {
        statistics::HistorySyncWriteProfileAdminStatistics::new(self.cmds)
    }
}
