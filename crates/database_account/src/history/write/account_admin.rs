use database::define_history_write_commands;

mod client_version;

define_history_write_commands!(HistoryWriteAccountAdmin);

impl<'a> HistoryWriteAccountAdmin<'a> {
    pub fn client_version(self) -> client_version::HistoryWriteAccountClientVersion<'a> {
        client_version::HistoryWriteAccountClientVersion::new(self.cmds)
    }
}
