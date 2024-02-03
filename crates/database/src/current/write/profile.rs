use diesel::{delete, insert_into, prelude::*, update, ExpressionMethods, QueryDsl};
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, Location, ProfileInternal, ProfileUpdateInternal, ProfileVersion};
use simple_backend_database::diesel_db::DieselDatabaseError;
use simple_backend_utils::current_unix_time;

use super::ConnectionProvider;
use crate::IntoDatabaseError;

mod data;
mod favorite;

define_write_commands!(CurrentWriteProfile, CurrentSyncWriteProfile);

impl<C: ConnectionProvider> CurrentSyncWriteProfile<C> {
    pub fn data(self) -> data::CurrentSyncWriteProfileData<C> {
        data::CurrentSyncWriteProfileData::new(self.cmds)
    }

    pub fn favorite(self) -> favorite::CurrentSyncWriteProfileFavorite<C> {
        favorite::CurrentSyncWriteProfileFavorite::new(self.cmds)
    }
}
