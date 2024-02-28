use diesel::{insert_into, prelude::*, update, ExpressionMethods, QueryDsl};
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, Location, ProfileInternal, ProfileStateInternal, ProfileUpdateInternal, ProfileVersion};
use simple_backend_database::diesel_db::DieselDatabaseError;

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_write_commands!(CurrentWriteProfileData, CurrentSyncWriteProfileData);

impl<C: ConnectionProvider> CurrentSyncWriteProfileData<C> {
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
            .into_db_error(id)
    }

    pub fn insert_profile_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_state::dsl::*;

        insert_into(profile_state)
            .values(account_id.eq(id.as_db_id()))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
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
            .set((name.eq(data),))
            .execute(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(())
    }

    pub fn profile_location(
        &mut self,
        id: AccountIdInternal,
        data: Location,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::profile_state::dsl::*;

        update(profile_state.find(id.as_db_id()))
            .set(data)
            .execute(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(())
    }

    pub fn profile_state(
        &mut self,
        id: AccountIdInternal,
        data: ProfileStateInternal,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::profile_state::dsl::*;

        update(profile_state.find(id.as_db_id()))
            .set(data)
            .execute(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(())
    }
}
