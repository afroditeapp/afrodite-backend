use server_data::define_server_data_read_commands;

pub mod news;

define_server_data_read_commands!(ReadCommandsAccountAdmin);
define_db_read_command!(ReadCommandsAccountAdmin);

impl<C: server_data::read::ReadCommandsProvider> ReadCommandsAccountAdmin<C> {
    pub fn news(self) -> news::ReadCommandsAccountNews<C> {
        news::ReadCommandsAccountNews::new(self.cmds)
    }
}
