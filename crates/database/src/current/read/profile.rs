


use simple_backend_database::diesel_db::{ConnectionProvider};

mod data;
mod favorite;

define_read_commands!(CurrentReadProfile, CurrentSyncReadProfile);

impl<C: ConnectionProvider> CurrentSyncReadProfile<C> {
    pub fn data(self) -> data::CurrentSyncReadProfileData<C> {
        data::CurrentSyncReadProfileData::new(self.cmds)
    }
    pub fn favorite(self) -> favorite::CurrentSyncReadProfileFavorite<C> {
        favorite::CurrentSyncReadProfileFavorite::new(self.cmds)
    }
}
