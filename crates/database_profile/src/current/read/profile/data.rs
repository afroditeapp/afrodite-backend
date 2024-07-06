use std::collections::HashMap;

use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, Location, Profile, ProfileAttributeFilterValue, ProfileAttributeValue, ProfileInternal, ProfileStateInternal, UnixTime
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
        let other_shared_state = self.read().common().state().other_shared_state(id)?;
        Ok(Profile::new(profile, attributes, other_shared_state.unlimited_likes))
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

    pub fn profile_last_seen_time(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<UnixTime>, DieselDatabaseError> {
        use crate::schema::profile::dsl::*;

        profile
            .filter(account_id.eq(id.as_db_id()))
            .select(last_seen_unix_time)
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
        let data: Vec<(i64, i64, Option<i64>)> = {
            use crate::schema::profile_attributes::dsl::*;
            profile_attributes
                .filter(account_id.eq(id.as_db_id()))
                .filter(attribute_value_part1.is_not_null())
                .select((
                    attribute_id,
                    attribute_value_part1.assume_not_null(),
                    attribute_value_part2,
                ))
                .load(self.conn())
                .change_context(DieselDatabaseError::Execute)?
        };

        let mut data: Vec<ProfileAttributeValue> = data
            .into_iter()
            .map(|(id, part1, part2)| {
                ProfileAttributeValue::new_not_number_list(
                    id as u16,
                    Some(part1 as u16)
                        .into_iter()
                        .chain(part2.map(|v| v as u16))
                        .collect()
                    )
            })
            .collect();

        let number_list_data: Vec<(i64, i64)> = {
            use crate::schema::profile_attributes_number_list::dsl::*;

            profile_attributes_number_list
                .filter(account_id.eq(id.as_db_id()))
                .select((
                    attribute_id,
                    attribute_value,
                ))
                .load(self.conn())
                .change_context(DieselDatabaseError::Execute)?
        };

        let mut number_list_attributes = HashMap::<u16, Vec<u16>>::new();
        for (id, value) in number_list_data {
            let values = number_list_attributes
                .entry(id as u16)
                .or_default();
            values.push(value as u16);
        }
        for (id, number_list) in number_list_attributes {
            data.push(ProfileAttributeValue::new_number_list(
                id,
                number_list
            ));
        }

        Ok(data)
    }

    /// Get profile attributes filter values which are set.
    pub fn profile_attribute_filters(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Vec<ProfileAttributeFilterValue>, DieselDatabaseError> {
        let data: Vec<(i64, Option<i64>, Option<i64>, bool)> = {
            use crate::schema::profile_attributes::dsl::*;

            profile_attributes
                .filter(account_id.eq(id.as_db_id()))
                .filter(filter_accept_missing_attribute.is_not_null())
                .select((
                    attribute_id,
                    filter_value_part1,
                    filter_value_part2,
                    filter_accept_missing_attribute.assume_not_null(),
                ))
                .load(self.conn())
                .change_context(DieselDatabaseError::Execute)?
        };

        let mut data: Vec<ProfileAttributeFilterValue> = data
            .into_iter()
            .map(|(id, part1, part2, accept_missing)| {
                ProfileAttributeFilterValue::new_not_number_list(
                    id as u16,
                    part1.map(|v| v as u16)
                        .into_iter()
                        .chain(part2.map(|v| v as u16))
                        .collect(),

                    accept_missing,
                )
            })
            .collect();

        let number_list_filters: Vec<(i64, i64)> = {
            use crate::schema::profile_attributes_number_list_filters::dsl::*;

            profile_attributes_number_list_filters
                .filter(account_id.eq(id.as_db_id()))
                .select((
                    attribute_id,
                    filter_value,
                ))
                .load(self.conn())
                .change_context(DieselDatabaseError::Execute)?
        };
        let mut number_list_attribute_filters = HashMap::<u16, Vec<u16>>::new();
        for (id, filter_value) in number_list_filters {
            let values = number_list_attribute_filters
                .entry(id as u16)
                .or_default();
            values.push(filter_value as u16);
        }
        for filter_value in &mut data {
            for (id, number_list) in &number_list_attribute_filters {
                if filter_value.id() == *id {
                    filter_value.set_number_list_filter_value(number_list.clone());
                }
            }
        }

        Ok(data)
    }
}
