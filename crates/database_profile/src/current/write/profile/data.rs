use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{
    delete, insert_into, prelude::*, update, upsert::excluded, ExpressionMethods, QueryDsl,
};
use error_stack::{Result, ResultExt};
use model_profile::{
    AccountIdInternal, Attribute, Location, ProfileAge, ProfileAttributeFilterValueUpdate, ProfileAttributeValueUpdate, ProfileAttributesInternal, ProfileEditedTime, ProfileFilteringSettingsUpdateValidated, ProfileInternal, ProfileStateInternal, ProfileUpdateValidated, ProfileVersion, SyncVersion, UnixTime
};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileData);

impl CurrentWriteProfileData<'_> {
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

        let edit_time = ProfileEditedTime::current_time();
        insert_into(profile_state)
            .values((
                account_id.eq(id.as_db_id()),
                profile_edited_unix_time.eq(edit_time),
            ))
            .execute(self.conn())
            .into_db_error(id)?;

        Ok(())
    }

    pub fn profile(
        &mut self,
        id: AccountIdInternal,
        data: &ProfileUpdateValidated,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::profile::dsl::*;

        update(profile.find(id.as_db_id()))
            .set((
                name.eq(&data.name),
                age.eq(&data.age),
                profile_text.eq(&data.ptext),
            ))
            .execute(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        Ok(())
    }

    pub fn profile_last_seen_time(
        &mut self,
        id: AccountIdInternal,
        data: Option<UnixTime>,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::profile::dsl::*;

        update(profile.find(id.as_db_id()))
            .set(last_seen_unix_time.eq(data))
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

    pub fn initial_profile_age(
        &mut self,
        id: AccountIdInternal,
        initial_age: ProfileAge,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::profile_state::dsl::*;

        let current_time = UnixTime::current_time();

        update(profile_state.find(id.as_db_id()))
            .set((
                profile_initial_age.eq(initial_age),
                profile_initial_age_set_unix_time.eq(current_time),
            ))
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

    pub fn upsert_profile_attributes_file_hash(
        &mut self,
        sha256_attribute_file_hash: &str,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_attributes_file_hash::dsl::*;

        insert_into(profile_attributes_file_hash)
            .values((row_type.eq(0), sha256_hash.eq(sha256_attribute_file_hash)))
            .on_conflict(row_type)
            .do_update()
            .set(sha256_hash.eq(sha256_attribute_file_hash))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    /// Update profile version, increment profile sync version and
    /// update profile edited time.
    pub fn required_changes_for_profile_update(
        &mut self,
        id: AccountIdInternal,
        data: ProfileVersion,
        time: ProfileEditedTime,
    ) -> Result<(), DieselDatabaseError> {

        // Update profile version
        {
            use crate::schema::profile::dsl::*;

            update(profile.find(id.as_db_id()))
                .set(version_uuid.eq(data))
                .execute(self.conn())
                .change_context(DieselDatabaseError::Execute)?;
        }

        // Increment profile sync version
        {
            use model::schema::profile_state::dsl::*;

            update(profile_state)
                .filter(account_id.eq(id.as_db_id()))
                .filter(profile_sync_version.lt(SyncVersion::MAX_VALUE))
                .set(profile_sync_version.eq(profile_sync_version + 1))
                .execute(self.conn())
                .into_db_error(())?;
        }

        // Update profile edited time
        {
            use crate::schema::profile_state::dsl::*;

            update(profile_state.find(id.as_db_id()))
                .set(profile_edited_unix_time.eq(time))
                .execute(self.conn())
                .change_context(DieselDatabaseError::Execute)?;
        }

        Ok(())
    }

    pub fn reset_profile_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_state::dsl::*;

        update(profile_state)
            .filter(account_id.eq(id.as_db_id()))
            .set(profile_sync_version.eq(0))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn update_profile_filtering_settings(
        &mut self,
        id: AccountIdInternal,
        settings: ProfileFilteringSettingsUpdateValidated,
        attributes: Option<&ProfileAttributesInternal>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_state::dsl::*;

        update(profile_state)
            .filter(account_id.eq(id.as_db_id()))
            .set((
                last_seen_time_filter.eq(settings.last_seen_time_filter),
                unlimited_likes_filter.eq(settings.unlimited_likes_filter),
                max_distance_km_filter.eq(settings.max_distance_km_filter),
                profile_created_time_filter.eq(settings.profile_created_filter),
                profile_edited_time_filter.eq(settings.profile_edited_filter),
                random_profile_order.eq(settings.random_profile_order),
            ))
            .execute(self.conn())
            .into_db_error(())?;

        self.upsert_profile_attribute_filters(id, settings.filters, attributes)?;

        Ok(())
    }

    pub fn upsert_profile_attributes(
        &mut self,
        id: AccountIdInternal,
        data: Vec<ProfileAttributeValueUpdate>,
        attributes: Option<&ProfileAttributesInternal>,
    ) -> Result<(), DieselDatabaseError> {
        // Using for loop here because this:
        // https://github.com/diesel-rs/diesel/discussions/3115
        // (SQLite does not support DEFAULT keyword when inserting data
        //  and Diesel seems to not support setting empty columns explicitly
        //  to NULL)

        for a in data {
            let is_number_list = attributes
                .and_then(|attributes| attributes.get_attribute(a.id))
                .map(|attribute: &Attribute| attribute.mode.is_number_list())
                .unwrap_or_default();

            if is_number_list {
                use model::schema::profile_attributes_number_list::dsl::*;

                delete(profile_attributes_number_list)
                    .filter(account_id.eq(id.as_db_id()))
                    .filter(attribute_id.eq(a.id))
                    .execute(self.conn())
                    .into_db_error(())?;

                let values: Vec<_> =
                    a.v.into_iter()
                        .map(|value| {
                            (
                                account_id.eq(id.as_db_id()),
                                attribute_id.eq(a.id),
                                attribute_value.eq(value as i64),
                            )
                        })
                        .collect();

                insert_into(profile_attributes_number_list)
                    .values(values)
                    .execute(self.conn())
                    .into_db_error(())?;
            } else {
                use model::schema::profile_attributes::dsl::*;

                insert_into(profile_attributes)
                    .values((
                        account_id.eq(id.as_db_id()),
                        attribute_id.eq(a.id),
                        attribute_value_part1.eq(a.v.first().copied().map(|v| v as i64)),
                        attribute_value_part2.eq(a.v.get(1).copied().map(|v| v as i64)),
                    ))
                    .on_conflict((account_id, attribute_id))
                    .do_update()
                    .set((
                        attribute_value_part1.eq(excluded(attribute_value_part1)),
                        attribute_value_part2.eq(excluded(attribute_value_part2)),
                    ))
                    .execute(self.conn())
                    .into_db_error(())?;
            }
        }

        Ok(())
    }

    fn upsert_profile_attribute_filters(
        &mut self,
        id: AccountIdInternal,
        data: Vec<ProfileAttributeFilterValueUpdate>,
        attributes: Option<&ProfileAttributesInternal>,
    ) -> Result<(), DieselDatabaseError> {
        // Using for loop here because this:
        // https://github.com/diesel-rs/diesel/discussions/3115
        // (SQLite does not support DEFAULT keyword when inserting data
        //  and Diesel seems to not support setting empty columns explicitly
        //  to NULL)

        for a in data {
            let is_number_list = attributes
                .and_then(|attributes| attributes.get_attribute(a.id))
                .map(|attribute: &Attribute| attribute.mode.is_number_list())
                .unwrap_or_default();

            let (part1, part2) = if is_number_list {
                use model::schema::profile_attributes_number_list_filters::dsl::*;

                delete(profile_attributes_number_list_filters)
                    .filter(account_id.eq(id.as_db_id()))
                    .filter(attribute_id.eq(a.id))
                    .execute(self.conn())
                    .into_db_error(())?;

                let values: Vec<_> = a
                    .filter_values
                    .into_iter()
                    .map(|value| {
                        (
                            account_id.eq(id.as_db_id()),
                            attribute_id.eq(a.id),
                            filter_value.eq(value as i64),
                        )
                    })
                    .collect();

                insert_into(profile_attributes_number_list_filters)
                    .values(values)
                    .execute(self.conn())
                    .into_db_error(())?;

                (None, None)
            } else {
                (
                    a.filter_values.first().copied().map(|v| v as i64),
                    a.filter_values.get(1).copied().map(|v| v as i64),
                )
            };

            {
                use model::schema::profile_attributes::dsl::*;

                insert_into(profile_attributes)
                    .values((
                        account_id.eq(id.as_db_id()),
                        attribute_id.eq(a.id),
                        filter_value_part1.eq(part1),
                        filter_value_part2.eq(part2),
                        filter_accept_missing_attribute.eq(a.accept_missing_attribute),
                    ))
                    .on_conflict((account_id, attribute_id))
                    .do_update()
                    .set((
                        filter_value_part1.eq(excluded(filter_value_part1)),
                        filter_value_part2.eq(excluded(filter_value_part2)),
                        filter_accept_missing_attribute
                            .eq(excluded(filter_accept_missing_attribute)),
                    ))
                    .execute(self.conn())
                    .into_db_error(())?;
            }
        }

        Ok(())
    }
}
