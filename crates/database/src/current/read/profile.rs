use diesel::{prelude::*, alias};
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, LocationIndexKey, ProfileInternal, Location, schema::profile_location, AccountIdDb};


use simple_backend_database::diesel_db::{ConnectionProvider, DieselDatabaseError};


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

    pub fn favorites(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Vec<AccountIdInternal>, DieselDatabaseError> {
        use crate::schema::favorite_profile;
        use crate::schema::account_id;

        let favorites = favorite_profile::table
            .inner_join(account_id::table.on(favorite_profile::favorite_account_id.eq(account_id::id)))
            .filter(favorite_profile::account_id.eq(id.as_db_id()))
            .order((favorite_profile::unix_time.asc(), favorite_profile::favorite_account_id.asc()))
            .select(AccountIdInternal::as_select())
            .load(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(favorites)
    }
}
