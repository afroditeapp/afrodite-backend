use database::define_history_read_commands;

mod client_version;

define_history_read_commands!(HistoryReadAccountAdmin);

impl<'a> HistoryReadAccountAdmin<'a> {
    pub fn client_version(self) -> client_version::HistoryReadAccountClientVersion<'a> {
        client_version::HistoryReadAccountClientVersion::new(self.cmds)
    }
}
