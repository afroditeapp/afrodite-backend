use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{delete, insert_into, prelude::*, update, upsert::excluded, ExpressionMethods, QueryDsl};
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, Attribute, Location, ProfileAttributeFilterValueUpdate, ProfileAttributeValueUpdate, ProfileAttributes, ProfileInternal, ProfileStateInternal, ProfileUpdateInternal, ProfileVersion, SyncVersion, UnixTime
};

use super::ConnectionProvider;
use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteProfileData, CurrentSyncWriteProfileData);

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
        data: &ProfileUpdateInternal,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::profile::dsl::*;

        update(profile.find(id.as_db_id()))
            .set((
                version_uuid.eq(data.version),
                name.eq(&data.new_data.name),
                age.eq(&data.new_data.age),
                profile_text.eq(&data.new_data.profile_text),
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

    pub fn increment_profile_attributes_sync_version_for_every_account(
        &mut self,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_state::dsl::*;

        update(profile_state)
            .filter(profile_attributes_sync_version.lt(SyncVersion::MAX_VALUE))
            .set(profile_attributes_sync_version.eq(profile_attributes_sync_version + 1))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn reset_profile_attributes_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_state::dsl::*;

        update(profile_state)
            .filter(account_id.eq(id.as_db_id()))
            .set(profile_attributes_sync_version.eq(0))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn upsert_profile_attributes(
        &mut self,
        id: AccountIdInternal,
        data: Vec<ProfileAttributeValueUpdate>,
        attributes: Option<&ProfileAttributes>,
    ) -> Result<(), DieselDatabaseError> {

        // Using for loop here because this:
        // https://github.com/diesel-rs/diesel/discussions/3115
        // (SQLite does not support DEFAULT keyword when inserting data
        //  and Diesel seems to not support setting empty columns explicitly
        //  to NULL)

        for a in data {
            let id_usize: usize = a.id.into();
            let is_number_list = attributes
                .and_then(|attributes| attributes.attributes.get(id_usize))
                .map(|attribute: &Attribute| attribute.mode.is_number_list())
                .unwrap_or_default();

            if is_number_list {
                use model::schema::profile_attributes_number_list::dsl::*;

                delete(profile_attributes_number_list)
                    .filter(account_id.eq(id.as_db_id()))
                    .filter(attribute_id.eq(a.id as i64))
                    .execute(self.conn())
                    .into_db_error(())?;

                let values: Vec<_> = a.values.into_iter().map(|value| {
                    (
                        account_id.eq(id.as_db_id()),
                        attribute_id.eq(a.id as i64),
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
                        attribute_id.eq(a.id as i64),
                        attribute_value_part1.eq(a.values.first().copied().map(|v| v as i64)),
                        attribute_value_part2.eq(a.values.get(1).copied().map(|v| v as i64)),
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

    pub fn upsert_profile_attribute_filters(
        &mut self,
        id: AccountIdInternal,
        data: Vec<ProfileAttributeFilterValueUpdate>,
        attributes: Option<&ProfileAttributes>,
    ) -> Result<(), DieselDatabaseError> {

        // Using for loop here because this:
        // https://github.com/diesel-rs/diesel/discussions/3115
        // (SQLite does not support DEFAULT keyword when inserting data
        //  and Diesel seems to not support setting empty columns explicitly
        //  to NULL)

        for a in data {
            let id_usize: usize = a.id.into();
            let is_number_list = attributes
                .and_then(|attributes| attributes.attributes.get(id_usize))
                .map(|attribute: &Attribute| attribute.mode.is_number_list())
                .unwrap_or_default();

            let (part1, part2) = if is_number_list {
                use model::schema::profile_attributes_number_list_filters::dsl::*;

                delete(profile_attributes_number_list_filters)
                    .filter(account_id.eq(id.as_db_id()))
                    .filter(attribute_id.eq(a.id as i64))
                    .execute(self.conn())
                    .into_db_error(())?;

                let values: Vec<_> = a.filter_values.into_iter().map(|value| {
                    (
                        account_id.eq(id.as_db_id()),
                        attribute_id.eq(a.id as i64),
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
                        attribute_id.eq(a.id as i64),
                        filter_value_part1.eq(part1),
                        filter_value_part2.eq(part2),
                        filter_accept_missing_attribute.eq(a.accept_missing_attribute),
                    ))
                    .on_conflict((account_id, attribute_id))
                    .do_update()
                    .set((
                        filter_value_part1.eq(excluded(filter_value_part1)),
                        filter_value_part2.eq(excluded(filter_value_part2)),
                        filter_accept_missing_attribute.eq(excluded(filter_accept_missing_attribute)),
                    ))
                    .execute(self.conn())
                    .into_db_error(())?;
            }
        }

        Ok(())
    }
}
