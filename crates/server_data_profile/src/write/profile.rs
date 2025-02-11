use database::current::read::GetDbReadCommandsCommon;
use database_profile::current::{read::GetDbReadCommandsProfile, write::GetDbWriteCommandsProfile};
use model_profile::{
    AccountIdInternal, Location, ProfileEditedTime, ProfileFilteringSettingsUpdateValidated, ProfileSearchAgeRangeValidated, ProfileStateInternal, ProfileUpdateValidated, ProfileVersion, ValidatedSearchGroups
};
use server_data::{
    app::GetConfig,
    cache::profile::UpdateLocationCacheState,
    define_cmd_wrapper_write,
    index::LocationWrite,
    read::DbRead,
    result::Result,
    write::DbTransaction,
    DataError, IntoDataError,
};
use tracing::info;

use crate::cache::{CacheReadProfile, CacheWriteProfile};

pub mod report;

define_cmd_wrapper_write!(WriteCommandsProfile);

impl<'a> WriteCommandsProfile<'a> {
    pub fn report(self) -> report::WriteCommandsProfileReport<'a> {
        report::WriteCommandsProfileReport::new(self.0)
    }
}

impl WriteCommandsProfile<'_> {
    pub async fn profile_update_location(
        &self,
        id: AccountIdInternal,
        coordinates: Location,
    ) -> Result<(), DataError> {
        let (location, max_distance, random_profile_order) = self
            .read_cache_profile_and_common(id.as_id(), |p, _| Ok((p.location.clone(), p.state.max_distance_km_filter, p.state.random_profile_order)))
            .await
            .into_data_error(id)?;

        db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile_location(id, coordinates)
        })?;

        let new_location_area = self.location().coordinates_to_area(coordinates, max_distance);
        self.location()
            .update_profile_location(id.as_id(), location.current_position.profile_location(), new_location_area.profile_location())
            .await?;

        let new_iterator_state = self
            .location_iterator()
            .new_iterator_state(
                &new_location_area,
                random_profile_order,
            );
        self.write_cache_profile(id, |p| {
            p.location.current_position = new_location_area;
            p.location.current_iterator = new_iterator_state;
            Ok(())
        })
        .await?;

        Ok(())
    }

    // TODO(refactor): New type for ProfileVersion::new_random() and
    //                 ProfileEditedTime::current_time().

    /// Updates [model::Profile].
    ///
    /// Updates also [model::ProfileSyncVersion].
    ///
    /// Check also
    /// [crate::write::profile_admin::profile_name_allowlist::WriteCommandsProfileAdminProfileNameAllowlist::moderate_profile_name]
    /// and from other `server_data_all`
    /// `UnlimitedLikesUpdate::update_unlimited_likes_value`
    /// as those also modifies the [model::Profile].
    pub async fn profile(
        &self,
        id: AccountIdInternal,
        data: ProfileUpdateValidated,
    ) -> Result<(), DataError> {
        let profile_data = data.clone();
        let config = self.config_arc().clone();
        let profile_version = ProfileVersion::new_random();
        let edit_time = ProfileEditedTime::current_time();
        let profile_text_moderation_state_update = db_transaction!(self, move |mut cmds| {
            let (name_update_detected, text_update_detected) = {
                let current_profile = cmds.read().profile().data().profile(id)?;
                (
                    current_profile.name != profile_data.name,
                    current_profile.ptext != profile_data.ptext,
                )
            };
            cmds.profile().data().profile(id, &profile_data)?;
            cmds.profile().data().upsert_profile_attributes(
                id,
                profile_data.attributes,
                config.profile_attributes(),
            )?;
            cmds.profile().data().required_changes_for_profile_update(id, profile_version, edit_time)?;
            if name_update_detected {
                cmds.profile()
                    .profile_name_allowlist()
                    .reset_profile_name_moderation_state(
                        id,
                        &profile_data.name,
                        config.profile_name_allowlist(),
                    )?;
            }
            let profile_text_moderation_state_update = if text_update_detected {
                Some(
                    cmds.profile()
                        .profile_text()
                        .reset_profile_text_moderation_state(
                            id,
                            profile_data.ptext.is_empty(),
                        )?,
                )
            } else {
                None
            };
            Ok(profile_text_moderation_state_update)
        })?;

        self.write_cache_profile(id.as_id(), |p| {
            data.update_to_profile(&mut p.data);
            data.update_to_attributes(&mut p.attributes);
            p.data.version_uuid = profile_version;
            p.state.profile_edited_time = edit_time;
            if let Some(update) = profile_text_moderation_state_update {
                p.state.profile_text_moderation_state = update;
            }
            Ok(())
        })
        .await
        .into_data_error(id)?;

        self.update_location_cache_profile(id).await?;

        Ok(())
    }

    async fn modify_profile_state(
        &self,
        id: AccountIdInternal,
        action: impl FnOnce(&mut ProfileStateInternal),
    ) -> Result<(), DataError> {
        let mut s = self
            .db_read(move |mut cmd| cmd.profile().data().profile_state(id))
            .await?;
        action(&mut s);
        let s_cloned = s.clone();
        db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile_state(id, s_cloned)?;
            cmds.read().common().account(id)
        })?;

        self.write_cache_profile(id.as_id(), |p| {
            p.state = s.into();
            Ok(())
        })
        .await
        .into_data_error(id)?;

        self.update_location_cache_profile(id).await?;

        Ok(())
    }

    pub async fn update_profile_filtering_settings(
        &self,
        id: AccountIdInternal,
        filters: ProfileFilteringSettingsUpdateValidated,
    ) -> Result<(), DataError> {
        let config = self.config_arc().clone();
        let filters_clone = filters.clone();
        let (new_filters, location) = db_transaction!(self, move |mut cmds| {
            cmds.profile().data().update_profile_filtering_settings(
                id,
                filters_clone,
                config.profile_attributes(),
            )?;
            let attribute_filters = cmds.read().profile().data().profile_attribute_filters(id)?;
            let location = cmds.read().profile().data().profile_location(id)?;
            Ok((attribute_filters, location))
        })?;

        self.write_cache_profile(id.as_id(), |p| {
            p.filters = new_filters;
            p.state.last_seen_time_filter = filters.last_seen_time_filter;
            p.state.unlimited_likes_filter = filters.unlimited_likes_filter;
            p.state.max_distance_km_filter = filters.max_distance_km_filter;
            p.state.profile_created_time_filter = filters.profile_created_filter;
            p.state.profile_edited_time_filter = filters.profile_edited_filter;
            p.state.random_profile_order = filters.random_profile_order;

            p.location.current_position = self.location().coordinates_to_area(location, filters.max_distance_km_filter);

            Ok(())
        })
        .await
        .into_data_error(id)?;

        Ok(())
    }

    pub async fn profile_name(&self, id: AccountIdInternal, data: String) -> Result<(), DataError> {
        let profile_data = data.clone();
        db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile_name(id, profile_data)
        })?;

        self.write_cache_profile(id.as_id(), |p| {
            p.data.name = data;
            Ok(())
        })
        .await
        .into_data_error(id)?;

        Ok(())
    }

    pub async fn insert_favorite_profile(
        &self,
        id: AccountIdInternal,
        favorite: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .favorite()
                .insert_favorite_profile(id, favorite)
        })
    }

    pub async fn remove_favorite_profile(
        &self,
        id: AccountIdInternal,
        favorite: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .favorite()
                .remove_favorite_profile(id, favorite)
        })
    }

    /// Updates the profile attributes sha256 and sync version for it for every
    /// account if needed.
    pub async fn update_profile_attributes_sha256_and_sync_versions(
        &self,
        sha256: String,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let current_hash = cmds.read().profile().data().attribute_file_hash()?;

            if current_hash.as_deref() != Some(&sha256) {
                info!(
                    "Profile attributes file hash changed from {:?} to {:?}",
                    current_hash,
                    Some(&sha256)
                );

                cmds.profile()
                    .data()
                    .upsert_profile_attributes_file_hash(&sha256)?;

                cmds.profile()
                    .data()
                    .increment_profile_attributes_sync_version_for_every_account()?;
            }

            Ok(())
        })
    }

    /// Only server WebSocket code should call this method.
    pub async fn reset_profile_attributes_sync_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .data()
                .reset_profile_attributes_sync_version(id)
        })
    }

    /// Only server WebSocket code should call this method.
    pub async fn reset_profile_sync_version(&self, id: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile().data().reset_profile_sync_version(id)
        })
    }

    pub async fn update_search_groups(
        &self,
        id: AccountIdInternal,
        search_groups: ValidatedSearchGroups,
    ) -> Result<(), DataError> {
        self.modify_profile_state(id, |s| s.search_group_flags = search_groups.into())
            .await
    }

    pub async fn update_search_age_range(
        &self,
        id: AccountIdInternal,
        range: ProfileSearchAgeRangeValidated,
    ) -> Result<(), DataError> {
        self.modify_profile_state(id, |s| {
            s.search_age_range_min = range.min();
            s.search_age_range_max = range.max();
        })
        .await
    }

    pub async fn benchmark_update_profile_bypassing_cache(
        &self,
        id: AccountIdInternal,
        data: ProfileUpdateValidated,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile(id, &data)
        })
    }

    pub async fn update_last_seen_time_from_cache_to_database(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        let last_seen_time = self
            .read_cache_profile_and_common(id, |p, _| Ok(p.last_seen_time_for_db()))
            .await?;

        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .data()
                .profile_last_seen_time(id, last_seen_time)
        })
    }

    pub async fn set_initial_profile_age_from_current_profile(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let profile = cmds.read().profile().data().profile(id)?;
            cmds.profile().data().initial_profile_age(id, profile.age)
        })
    }
}
