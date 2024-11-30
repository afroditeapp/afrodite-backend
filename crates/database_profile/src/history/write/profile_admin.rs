use database::define_history_write_commands;

mod statistics;

define_history_write_commands!(HistoryWriteProfileAdmin);

impl<'a> HistoryWriteProfileAdmin<'a> {
    pub fn statistics(self) -> statistics::HistoryWriteProfileAdminStatistics<'a> {
        statistics::HistoryWriteProfileAdminStatistics::new(self.cmds)
    }
}
