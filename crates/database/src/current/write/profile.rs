use diesel::{insert_into, prelude::*, update, ExpressionMethods, QueryDsl};
use error_stack::Result;
use model::{
    AccountIdInternal, LocationIndexKey, ProfileInternal, ProfileUpdateInternal, ProfileVersion,
};
use utils::IntoReportExt;

use crate::{diesel::DieselDatabaseError, IntoDatabaseError};

use super::ConnectionProvider;

define_write_commands!(CurrentWriteProfile, CurrentSyncWriteProfile);

impl<'a, C: ConnectionProvider> CurrentSyncWriteProfile<C> {
    pub fn insert_profile(
        &'a mut self,
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

    pub fn profile(
        &'a mut self,
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
            .into_error(DieselDatabaseError::Execute)?;

        Ok(())
    }

    pub fn profile_location(
        &'a mut self,
        id: AccountIdInternal,
        data: LocationIndexKey,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::profile::dsl::*;

        update(profile.find(id.as_db_id()))
            .set((
                location_key_x.eq(data.x as i64),
                location_key_y.eq(data.y as i64),
            ))
            .execute(self.conn())
            .into_error(DieselDatabaseError::Execute)?;

        Ok(())
    }
}
