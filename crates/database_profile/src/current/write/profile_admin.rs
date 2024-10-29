use database::{define_current_write_commands, ConnectionProvider};

mod profile_name_allowlist;

define_current_write_commands!(CurrentWriteProfileAdmin, CurrentSyncWriteProfileAdmin);

impl<C: ConnectionProvider> CurrentSyncWriteProfileAdmin<C> {
    pub fn profile_name_allowlist(self) -> profile_name_allowlist::CurrentWriteProfileAdminProfileNameAllowlist<C> {
        profile_name_allowlist::CurrentWriteProfileAdminProfileNameAllowlist::new(self.cmds)
    }
}
