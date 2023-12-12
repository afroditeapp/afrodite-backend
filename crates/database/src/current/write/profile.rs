use diesel::{insert_into, prelude::*, update, ExpressionMethods, QueryDsl, delete};
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, ProfileInternal, ProfileUpdateInternal, ProfileVersion, Location,
};
use simple_backend_database::diesel_db::DieselDatabaseError;
use simple_backend_utils::current_unix_time;


use crate::IntoDatabaseError;

use super::ConnectionProvider;

define_write_commands!(CurrentWriteProfile, CurrentSyncWriteProfile);

impl<C: ConnectionProvider> CurrentSyncWriteProfile<C> {
    pub fn insert_profile(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<ProfileInternal, DieselDatabaseError> {
        use model::schema::profile::dsl::*;

        let version = ProfileVersion::new_random();
        insert_into(profile)
            .values((account_id.eq(id.as_db_id()), version_uuid.eq(version)))
            .returning(ProfileInternal::as_returning())
            .get_result(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)
    }

    pub fn insert_profile_location(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Location, DieselDatabaseError> {
        use model::schema::profile_location::dsl::*;

        insert_into(profile_location)
            .values(account_id.eq(id.as_db_id()))
            .returning(Location::as_returning())
            .get_result(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)
    }

    pub fn profile(
        &mut self,
        id: AccountIdInternal,
        data: ProfileUpdateInternal,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::profile::dsl::*;

        update(profile.find(id.as_db_id()))
            .set((
                version_uuid.eq(data.version),
                profile_text.eq(data.new_data.profile_text),
            ))
            .execute(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(())
    }

    pub fn profile_name(
        &mut self,
        id: AccountIdInternal,
        data: String,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::profile::dsl::*;

        update(profile.find(id.as_db_id()))
            .set((
                name.eq(data),
            ))
            .execute(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(())
    }

    pub fn profile_location(
        &mut self,
        id: AccountIdInternal,
        data: Location,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::profile_location::dsl::*;

        update(profile_location.find(id.as_db_id()))
            .set((
                latitude.eq(data.latitude),
                longitude.eq(data.longitude),
            ))
            .execute(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(())
    }

    pub fn insert_favorite_profile(
        &mut self,
        id: AccountIdInternal,
        favorite: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::favorite_profile::dsl::*;

        let time = current_unix_time();

        insert_into(favorite_profile)
            .values((
                account_id.eq(id.as_db_id()),
                favorite_account_id.eq(favorite.as_db_id()),
                unix_time.eq(time)
            ))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }

    pub fn remove_favorite_profile(
        &mut self,
        id: AccountIdInternal,
        favorite: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::favorite_profile::dsl::*;

        delete(favorite_profile)
            .filter(account_id.eq(id.as_db_id()))
            .filter(favorite_account_id.eq(favorite.as_db_id()))
            .execute(self.conn())
            .into_db_error(DieselDatabaseError::Execute, id)?;

        Ok(())
    }
}
