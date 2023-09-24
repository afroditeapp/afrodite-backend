use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, LocationIndexKey, ProfileInternal, Location, schema::profile_location};


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
            .change_context(DieselDatabaseError::Execute)
    }

    pub fn profile_location(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Location, DieselDatabaseError> {
        use crate::schema::profile_location::dsl::*;

        let (lat, lon) = profile_location
            .filter(account_id.eq(id.as_db_id()))
            .select((latitude, longitude))
            .first::<(f64, f64)>(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(Location {
            latitude: lat,
            longitude: lon,
        })
    }
}
