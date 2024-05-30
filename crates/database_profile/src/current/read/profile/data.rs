use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, Location, Profile, ProfileAttributeFilterValue, ProfileAttributeValue,
    ProfileInternal, ProfileStateInternal,
};

define_current_read_commands!(CurrentReadProfileData, CurrentSyncReadProfileData);

impl<C: ConnectionProvider> CurrentSyncReadProfileData<C> {
    pub fn profile_internal(
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

    pub fn profile(&mut self, id: AccountIdInternal) -> Result<Profile, DieselDatabaseError> {
        let profile = self.profile_internal(id)?;
        let attributes = self.profile_attribute_values(id)?;
        Ok(Profile::new(profile, attributes))
    }

    pub fn profile_location(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Location, DieselDatabaseError> {
        use crate::schema::profile_state::dsl::*;

        profile_state
            .filter(account_id.eq(id.as_db_id()))
            .select(Location::as_select())
            .first(self.conn())
            .change_context(DieselDatabaseError::Execute)
    }

    pub fn profile_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<ProfileStateInternal, DieselDatabaseError> {
        use crate::schema::profile_state::dsl::*;

        profile_state
            .filter(account_id.eq(id.as_db_id()))
            .select(ProfileStateInternal::as_select())
            .first(self.conn())
            .change_context(DieselDatabaseError::Execute)
    }

    pub fn attribute_file_hash(&mut self) -> Result<Option<String>, DieselDatabaseError> {
        use crate::schema::profile_attributes_file_hash::dsl::*;

        profile_attributes_file_hash
            .filter(row_type.eq(0))
            .select(sha256_hash)
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
    }

    /// Get profile attributes values for attributes which are set.
    pub fn profile_attribute_values(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Vec<ProfileAttributeValue>, DieselDatabaseError> {
        use crate::schema::profile_attributes::dsl::*;

        let data: Vec<(i64, i64, Option<i64>)> = profile_attributes
            .filter(account_id.eq(id.as_db_id()))
            .filter(attribute_value_part1.is_not_null())
            .select((
                attribute_id,
                attribute_value_part1.assume_not_null(),
                attribute_value_part2,
            ))
            .load(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        let data = data
            .into_iter()
            .map(|(id, part1, part2)| {
                ProfileAttributeValue::new(id as u16, part1 as u16, part2.map(|v| v as u16))
            })
            .collect();

        Ok(data)
    }

    /// Get profile attributes filter values which are set.
    pub fn profile_attribute_filters(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Vec<ProfileAttributeFilterValue>, DieselDatabaseError> {
        use crate::schema::profile_attributes::dsl::*;

        let data: Vec<(i64, Option<i64>, Option<i64>, bool)> = profile_attributes
            .filter(account_id.eq(id.as_db_id()))
            .filter(filter_accept_missing_attribute.is_not_null())
            .select((
                attribute_id,
                filter_value_part1,
                filter_value_part2,
                filter_accept_missing_attribute.assume_not_null(),
            ))
            .load(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        let data = data
            .into_iter()
            .map(|(id, part1, part2, accept_missing)| {
                ProfileAttributeFilterValue::new(
                    id as u16,
                    part1.map(|v| v as u16),
                    part2.map(|v| v as u16),
                    accept_missing,
                )
            })
            .collect();

        Ok(data)
    }
}
