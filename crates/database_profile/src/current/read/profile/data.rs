use std::collections::HashMap;

use database::{
    DieselDatabaseError, current::read::GetDbReadCommandsCommon, define_current_read_commands,
};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model_profile::{
    AccountIdInternal, AttributeId, GetMyProfileResult, InitialProfileAge, LastSeenTime,
    LastSeenUnixTime, Location, Profile, ProfileAge, ProfileAttributeFilterValue,
    ProfileAttributeValue, ProfileInternal, ProfileStateInternal,
    ProfileStringModerationContentType, UnixTime,
};

use crate::current::read::GetDbReadCommandsProfile;

define_current_read_commands!(CurrentReadProfileData);

impl CurrentReadProfileData<'_> {
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
        let profile_name_moderation_state = self
            .read()
            .profile()
            .moderation()
            .profile_name_moderation_state(id)?;
        let profile_text_moderation_state = self
            .read()
            .profile()
            .moderation()
            .profile_text_moderation_state(id)?;
        Ok(Profile::new(
            profile,
            profile_name_moderation_state,
            profile_text_moderation_state,
            attributes,
            other_shared_state.unlimited_likes,
        ))
    }

    pub fn my_profile(
        &mut self,
        id: AccountIdInternal,
        last_seen_time: Option<LastSeenTime>,
    ) -> Result<GetMyProfileResult, DieselDatabaseError> {
        let profile = self.profile_internal(id)?;
        let profile_version = profile.version_uuid;
        let profile_state = self.profile_state(id)?;
        let attributes = self.profile_attribute_values(id)?;
        let other_shared_state = self.read().common().state().other_shared_state(id)?;
        let profile_name_moderation_state = self
            .read()
            .profile()
            .moderation()
            .profile_moderation_info(id, ProfileStringModerationContentType::ProfileName)?;
        let profile_text_moderation_state = self
            .read()
            .profile()
            .moderation()
            .profile_moderation_info(id, ProfileStringModerationContentType::ProfileText)?;
        let p = Profile::new(
            profile,
            profile_name_moderation_state
                .as_ref()
                .map(|v| v.state.into()),
            profile_text_moderation_state
                .as_ref()
                .map(|v| v.state.into()),
            attributes,
            other_shared_state.unlimited_likes,
        );
        let r = GetMyProfileResult {
            p,
            lst: last_seen_time,
            v: profile_version,
            sv: profile_state.profile_sync_version,
            name_moderation_info: profile_name_moderation_state,
            text_moderation_info: profile_text_moderation_state,
        };
        Ok(r)
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
    ) -> Result<LastSeenUnixTime, DieselDatabaseError> {
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
        let attribute_values_data: Vec<(AttributeId, i64)> = {
            use crate::schema::profile_attributes_value_list::dsl::*;

            profile_attributes_value_list
                .filter(account_id.eq(id.as_db_id()))
                .select((attribute_id, attribute_value))
                .load(self.conn())
                .change_context(DieselDatabaseError::Execute)?
        };

        let mut attributes = HashMap::<AttributeId, Vec<u32>>::new();
        for (id, value) in attribute_values_data {
            let values = attributes.entry(id).or_default();
            values.push(value as u32);
        }

        let mut data: Vec<ProfileAttributeValue> = attributes
            .into_iter()
            .map(|(id, data)| ProfileAttributeValue::new(id, data))
            .collect();

        data.sort_by_key(|v| v.id());

        Ok(data)
    }

    /// Get profile attributes filter values which are set.
    pub fn profile_attribute_filters(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Vec<ProfileAttributeFilterValue>, DieselDatabaseError> {
        let data: Vec<(AttributeId, bool, bool)> = {
            use crate::schema::profile_attributes_filter_settings::dsl::*;

            profile_attributes_filter_settings
                .filter(account_id.eq(id.as_db_id()))
                .select((
                    attribute_id,
                    filter_accept_missing_attribute,
                    filter_use_logical_operator_and,
                ))
                .load(self.conn())
                .change_context(DieselDatabaseError::Execute)?
        };

        #[derive(Default)]
        struct FilterValues {
            wanted: Vec<u32>,
            unwanted: Vec<u32>,
        }

        let mut all_values = HashMap::<AttributeId, FilterValues>::new();

        {
            use crate::schema::profile_attributes_filter_list_wanted::dsl::*;

            let attribute_filters_data_wanted: Vec<(AttributeId, i64)> =
                profile_attributes_filter_list_wanted
                    .filter(account_id.eq(id.as_db_id()))
                    .select((attribute_id, filter_value))
                    .load(self.conn())
                    .change_context(DieselDatabaseError::Execute)?;

            for (id, value) in attribute_filters_data_wanted {
                let values = all_values.entry(id).or_default();
                values.wanted.push(value as u32);
            }
        }

        {
            use crate::schema::profile_attributes_filter_list_unwanted::dsl::*;

            let attribute_filters_data_unwanted: Vec<(AttributeId, i64)> =
                profile_attributes_filter_list_unwanted
                    .filter(account_id.eq(id.as_db_id()))
                    .select((attribute_id, filter_value))
                    .load(self.conn())
                    .change_context(DieselDatabaseError::Execute)?;

            for (id, value) in attribute_filters_data_unwanted {
                let values = all_values.entry(id).or_default();
                values.unwanted.push(value as u32);
            }
        };

        let mut data: Vec<ProfileAttributeFilterValue> = data
            .into_iter()
            .map(|(id, accept_missing_attribute, use_logical_operator_and)| {
                let values = all_values.remove(&id).unwrap_or_default();
                ProfileAttributeFilterValue::new(
                    id,
                    values.wanted,
                    values.unwanted,
                    accept_missing_attribute,
                    use_logical_operator_and,
                )
            })
            .collect();

        data.sort_by_key(|v| v.id());

        Ok(data)
    }

    /// Returns Ok(None) if the initial profile age is not yet set.
    /// The age is set when complete initial setup command runs.
    pub fn initial_profile_age(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<InitialProfileAge>, DieselDatabaseError> {
        use crate::schema::profile_state::dsl::*;

        let r: (Option<ProfileAge>, Option<UnixTime>) = profile_state
            .filter(account_id.eq(id.as_db_id()))
            .select((initial_profile_age, initial_profile_age_set_unix_time))
            .first(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        if let (Some(age), Some(time)) = r {
            Ok(Some(InitialProfileAge {
                initial_profile_age: age,
                initial_profile_age_set_unix_time: time,
            }))
        } else {
            Ok(None)
        }
    }
}
