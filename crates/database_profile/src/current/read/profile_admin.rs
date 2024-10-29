use database::{define_current_read_commands, ConnectionProvider};

mod profile_name_allowlist;

define_current_read_commands!(CurrentReadProfileAdmin, CurrentSyncReadProfileAdmin);

impl<C: ConnectionProvider> CurrentSyncReadProfileAdmin<C> {
    pub fn profile_name_allowlist(self) -> profile_name_allowlist::CurrentSyncReadProfileNameAllowlist<C> {
        profile_name_allowlist::CurrentSyncReadProfileNameAllowlist::new(self.cmds)
    }
}
