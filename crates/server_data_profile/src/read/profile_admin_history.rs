use server_data::{define_server_data_read_commands, read::ReadCommandsProvider};

mod statistics;

define_server_data_read_commands!(ReadCommandsProfileAdminHistory);

impl<C: ReadCommandsProvider> ReadCommandsProfileAdminHistory<C> {
    pub fn statistics(self) -> statistics::ReadCommandsProfileAdminHistoryStatistics<C> {
        statistics::ReadCommandsProfileAdminHistoryStatistics::new(self.cmds)
    }
}
