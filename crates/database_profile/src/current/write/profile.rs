use database::define_current_write_commands;

use super::ConnectionProvider;

mod data;
mod favorite;
mod profile_name_allowlist;
mod profile_text;

define_current_write_commands!(CurrentWriteProfile, CurrentSyncWriteProfile);

impl<C: ConnectionProvider> CurrentSyncWriteProfile<C> {
    pub fn data(self) -> data::CurrentSyncWriteProfileData<C> {
        data::CurrentSyncWriteProfileData::new(self.cmds)
    }

    pub fn favorite(self) -> favorite::CurrentSyncWriteProfileFavorite<C> {
        favorite::CurrentSyncWriteProfileFavorite::new(self.cmds)
    }

    pub fn profile_name_allowlist(self) -> profile_name_allowlist::CurrentSyncWriteProfileNameAllowlist<C> {
        profile_name_allowlist::CurrentSyncWriteProfileNameAllowlist::new(self.cmds)
    }

    pub fn profile_text(self) -> profile_text::CurrentSyncWriteProfileText<C> {
        profile_text::CurrentSyncWriteProfileText::new(self.cmds)
    }
}
