use server_data::define_server_data_write_commands;
use server_data::write::WriteCommandsProvider;

pub mod profile_name_allowlist;

define_server_data_write_commands!(WriteCommandsProfileAdmin);

impl<C: WriteCommandsProvider> WriteCommandsProfileAdmin<C> {
    pub fn profile_name_allowlist(self) -> profile_name_allowlist::WriteCommandsProfileAdminProfileNameAllowlist<C> {
        profile_name_allowlist::WriteCommandsProfileAdminProfileNameAllowlist::new(self.cmds)
    }
}
