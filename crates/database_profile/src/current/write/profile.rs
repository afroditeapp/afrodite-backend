use database::define_current_write_commands;

use super::ConnectionProvider;

mod data;
mod favorite;
mod setup;

define_current_write_commands!(CurrentWriteProfile, CurrentSyncWriteProfile);

impl<C: ConnectionProvider> CurrentSyncWriteProfile<C> {
    pub fn data(self) -> data::CurrentSyncWriteProfileData<C> {
        data::CurrentSyncWriteProfileData::new(self.cmds)
    }

    pub fn favorite(self) -> favorite::CurrentSyncWriteProfileFavorite<C> {
        favorite::CurrentSyncWriteProfileFavorite::new(self.cmds)
    }

    pub fn setup(self) -> setup::CurrentSyncWriteProfileSetup<C> {
        setup::CurrentSyncWriteProfileSetup::new(self.cmds)
    }
}
