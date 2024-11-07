use database::{define_current_read_commands, ConnectionProvider};

mod profile_name_allowlist;
mod profile_text;

define_current_read_commands!(CurrentReadProfileAdmin, CurrentSyncReadProfileAdmin);

impl<C: ConnectionProvider> CurrentSyncReadProfileAdmin<C> {
    pub fn profile_name_allowlist(self) -> profile_name_allowlist::CurrentSyncReadProfileNameAllowlist<C> {
        profile_name_allowlist::CurrentSyncReadProfileNameAllowlist::new(self.cmds)
    }

    pub fn profile_text(self) -> profile_text::CurrentSyncReadProfileText<C> {
        profile_text::CurrentSyncReadProfileText::new(self.cmds)
    }
}
