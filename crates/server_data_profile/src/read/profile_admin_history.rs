use server_data::define_cmd_wrapper_read;

mod statistics;

define_cmd_wrapper_read!(ReadCommandsProfileAdminHistory);

impl<'a> ReadCommandsProfileAdminHistory<'a> {
    pub fn statistics(self) -> statistics::ReadCommandsProfileAdminHistoryStatistics<'a> {
        statistics::ReadCommandsProfileAdminHistoryStatistics::new(self.0)
    }
}
