use database::{define_history_read_commands, ConnectionProvider};
mod statistics;

define_history_read_commands!(HistoryReadProfileAdmin, HistorySyncReadProfileAdmin);

impl<C: ConnectionProvider> HistorySyncReadProfileAdmin<C> {
    pub fn statistics(
        self,
    ) -> statistics::HistorySyncReadProfileAdminStatistics<C> {
        statistics::HistorySyncReadProfileAdminStatistics::new(self.cmds)
    }
}
