use database::define_current_read_commands;
use database::ConnectionProvider;

mod data;
mod favorite;

define_current_read_commands!(CurrentReadProfile, CurrentSyncReadProfile);

impl<C: ConnectionProvider> CurrentSyncReadProfile<C> {
    pub fn data(self) -> data::CurrentSyncReadProfileData<C> {
        data::CurrentSyncReadProfileData::new(self.cmds)
    }
    pub fn favorite(self) -> favorite::CurrentSyncReadProfileFavorite<C> {
        favorite::CurrentSyncReadProfileFavorite::new(self.cmds)
    }
}
