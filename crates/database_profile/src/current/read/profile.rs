use database::{define_current_read_commands, ConnectionProvider};

mod data;
mod favorite;
mod profile_name_allowlist;

define_current_read_commands!(CurrentReadProfile, CurrentSyncReadProfile);

impl<C: ConnectionProvider> CurrentSyncReadProfile<C> {
    pub fn data(self) -> data::CurrentSyncReadProfileData<C> {
        data::CurrentSyncReadProfileData::new(self.cmds)
    }
    pub fn favorite(self) -> favorite::CurrentSyncReadProfileFavorite<C> {
        favorite::CurrentSyncReadProfileFavorite::new(self.cmds)
    }
    pub fn profile_name_allowlist(self) -> profile_name_allowlist::CurrentSyncReadProfileNameAllowlist<C> {
        profile_name_allowlist::CurrentSyncReadProfileNameAllowlist::new(self.cmds)
    }
}
