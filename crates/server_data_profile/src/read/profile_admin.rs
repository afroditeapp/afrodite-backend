use server_data::define_server_data_read_commands;
use server_data::read::ReadCommandsProvider;

mod profile_name_allowlist;

define_server_data_read_commands!(ReadCommandsProfileAdmin);

impl<C: ReadCommandsProvider> ReadCommandsProfileAdmin<C> {
    pub fn profile_name_allowlist(self) -> profile_name_allowlist::ReadCommandsProfileNameAllowlist<C> {
        profile_name_allowlist::ReadCommandsProfileNameAllowlist::new(self.cmds)
    }
}
