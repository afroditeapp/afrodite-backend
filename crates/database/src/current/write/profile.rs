use diesel::{insert_into, prelude::*, update, ExpressionMethods, QueryDsl};
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, ProfileInternal, ProfileUpdateInternal, ProfileVersion, Location,
};


use super::ConnectionProvider;
use crate::{diesel::DieselDatabaseError, IntoDatabaseError};

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
}
