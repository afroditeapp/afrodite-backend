use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, LocationIndexKey, ProfileInternal};
use utils::IntoReportExt;

use crate::diesel::{ConnectionProvider, DieselDatabaseError};

define_read_commands!(CurrentReadProfile, CurrentSyncReadProfile);

impl<C: ConnectionProvider> CurrentSyncReadProfile<C> {
    pub fn profile(
        &mut self,
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
        &mut self,
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
