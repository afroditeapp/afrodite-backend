use server_data::define_server_data_write_commands;

mod news;

define_server_data_write_commands!(WriteCommandsAccountAdmin);

impl<C: server_data::write::WriteCommandsProvider> WriteCommandsAccountAdmin<C> {
    pub fn news(self) -> news::WriteCommandsAccountNewsAdmin<C> {
        news::WriteCommandsAccountNewsAdmin::new(self.cmds)
    }
}
