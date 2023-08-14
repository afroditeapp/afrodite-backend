use async_trait::async_trait;
use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, LocationIndexKey, ProfileInternal, ProfileVersion};
use utils::IntoReportExt;

use crate::{
    current::read::SqliteReadCommands,
    diesel::DieselDatabaseError,
    sqlite::{SqliteDatabaseError, SqliteSelectJson},
};

define_read_commands!(CurrentReadProfile, CurrentSyncReadProfile);

impl<'a> CurrentSyncReadProfile<'a> {
    pub fn profile(
        &'a mut self,
        id: AccountIdInternal,
    ) -> Result<ProfileInternal, DieselDatabaseError> {
        use crate::schema::profile::dsl::*;

        profile
            .filter(account_id.eq(id.as_db_id()))
            .select(ProfileInternal::as_select())
            .first(self.conn())
            .into_error(DieselDatabaseError::Execute)
    }

    pub fn location_index_key(
        &'a mut self,
        id: AccountIdInternal,
    ) -> Result<LocationIndexKey, DieselDatabaseError> {
        use crate::schema::profile::dsl::*;

        let (x, y) = profile
            .filter(account_id.eq(id.as_db_id()))
            .select((location_key_x, location_key_y))
            .first::<(i64, i64)>(self.conn())
            .into_error(DieselDatabaseError::Execute)?;

        Ok(LocationIndexKey {
            x: x as u16,
            y: y as u16,
        })
    }
}
