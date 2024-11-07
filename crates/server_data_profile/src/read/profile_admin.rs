use server_data::define_server_data_read_commands;
use server_data::read::ReadCommandsProvider;

mod profile_name_allowlist;
mod profile_text;

define_server_data_read_commands!(ReadCommandsProfileAdmin);

impl<C: ReadCommandsProvider> ReadCommandsProfileAdmin<C> {
    pub fn profile_name_allowlist(self) -> profile_name_allowlist::ReadCommandsProfileNameAllowlist<C> {
        profile_name_allowlist::ReadCommandsProfileNameAllowlist::new(self.cmds)
    }

    pub fn profile_text(self) -> profile_text::ReadCommandsProfileText<C> {
        profile_text::ReadCommandsProfileText::new(self.cmds)
    }
}
