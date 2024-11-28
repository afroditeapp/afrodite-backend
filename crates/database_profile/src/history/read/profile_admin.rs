use database::define_history_read_commands;
mod statistics;

define_history_read_commands!(HistoryReadProfileAdmin);

impl<'a> HistoryReadProfileAdmin<'a> {
    pub fn statistics(
        self,
    ) -> statistics::HistoryReadProfileAdminStatistics<'a> {
        statistics::HistoryReadProfileAdminStatistics::new(self.cmds)
    }
}
