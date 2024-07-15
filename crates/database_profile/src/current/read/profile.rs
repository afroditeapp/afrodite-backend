use database::{define_current_read_commands, ConnectionProvider};

mod data;
mod favorite;
mod setup;

define_current_read_commands!(CurrentReadProfile, CurrentSyncReadProfile);

impl<C: ConnectionProvider> CurrentSyncReadProfile<C> {
    pub fn data(self) -> data::CurrentSyncReadProfileData<C> {
        data::CurrentSyncReadProfileData::new(self.cmds)
    }
    pub fn favorite(self) -> favorite::CurrentSyncReadProfileFavorite<C> {
        favorite::CurrentSyncReadProfileFavorite::new(self.cmds)
    }
    pub fn setup(self) -> setup::CurrentSyncReadProfileSetup<C> {
        setup::CurrentSyncReadProfileSetup::new(self.cmds)
    }
}
