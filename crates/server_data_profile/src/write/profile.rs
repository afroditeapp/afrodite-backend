use model_profile::{
    AccountIdInternal, Location, ProfileAttributeFilterListUpdateValidated, ProfileSearchAgeRangeValidated, ProfileStateInternal, ProfileUpdateInternal, ValidatedSearchGroups
};
use server_data::{
    app::GetConfig, cache::profile::UpdateLocationCacheState, define_cmd_wrapper_write, result::Result, DataError, IntoDataError,
    index::{location::LocationIndexIteratorState, LocationWrite},
};
use tracing::info;
use crate::{cache::{CacheReadProfile, CacheWriteProfile}, read::DbReadProfile};

use super::DbTransactionProfile;

define_cmd_wrapper_write!(WriteCommandsProfile);

impl WriteCommandsProfile<'_> {
    pub async fn profile_update_location(
        &self,
        id: AccountIdInternal,
        coordinates: Location,
    ) -> Result<(), DataError> {
        let location = self
            .read_cache_profile_and_common(id.as_id(), |p, _| {
                Ok(p.location.clone())
            })
            .await
            .into_data_error(id)?;

        let new_location_key = self.location().coordinates_to_key(&coordinates);
        db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile_location(id, coordinates)
        })?;

        self.location()
            .update_profile_location(id.as_id(), location.current_position, new_location_key)
            .await?;

        let new_iterator_state = self
            .location_iterator()
            .reset_iterator(LocationIndexIteratorState::new(), new_location_key);
        self.write_cache_profile(id, |p| {
            p.location.current_position = new_location_key;
            p.location.current_iterator = new_iterator_state;
            Ok(())
        })
        .await?;

        Ok(())
    }

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
        data: ProfileUpdateInternal,
    ) -> Result<(), DataError> {
        let profile_data = data.clone();
        let config = self.config_arc().clone();
        let profile_text_moderation_state_update = db_transaction!(self, move |mut cmds| {
            let (name_update_detected, text_update_detected) = {
                let current_profile = cmds.read().profile().data().profile(id)?;
                (
                    current_profile.name != profile_data.new_data.name,
                    current_profile.ptext != profile_data.new_data.ptext
                )
            };
            cmds.profile().data().profile(id, &profile_data)?;
            cmds.profile()
                .data()
                .upsert_profile_attributes(id, profile_data.new_data.attributes, config.profile_attributes())?;
            cmds.profile().data().increment_profile_sync_version(id)?;
            if name_update_detected {
                cmds.profile()
                    .profile_name_allowlist()
                    .reset_profile_name_moderation_state(
                        id,
                        &profile_data.new_data.name,
                        config.profile_name_allowlist(),
                    )?;
            }
            let profile_text_moderation_state_update = if text_update_detected {
                Some(
                    cmds.profile()
                        .profile_text()
                        .reset_profile_text_moderation_state(
                            id,
                            profile_data.new_data.ptext.is_empty()
                        )?
                )
            } else {
                None
            };
            Ok(profile_text_moderation_state_update)
        })?;

        self
            .write_cache_profile(id.as_id(), |p| {
                data.new_data.update_to_profile(&mut p.data);
                data.new_data.update_to_attributes(&mut p.attributes);
                p.data.version_uuid = data.version;
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

        self
            .write_cache_profile(id.as_id(), |p| {
                p.state = s.into();
                Ok(())
            })
            .await
            .into_data_error(id)?;

        self.update_location_cache_profile(id).await?;

        Ok(())
    }

    pub async fn update_profile_attribute_filters(
        &self,
        id: AccountIdInternal,
        filters: ProfileAttributeFilterListUpdateValidated,
    ) -> Result<(), DataError> {
        let config = self.config_arc().clone();
        let new_filters = db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .data()
                .upsert_profile_attribute_filters(id, filters.filters, config.profile_attributes())?;
            cmds.profile()
                .data()
                .update_last_seen_time_filter(id, filters.last_seen_time_filter)?;
            cmds.profile()
                .data()
                .update_unlimited_likes_filter(id, filters.unlimited_likes_filter)?;
            cmds.read().profile().data().profile_attribute_filters(id)
        })?;

        self
            .write_cache_profile(id.as_id(), |p| {
                p.filters = new_filters;
                p.state.last_seen_time_filter = filters.last_seen_time_filter;
                p.state.unlimited_likes_filter = filters.unlimited_likes_filter;
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

        self
            .write_cache_profile(id.as_id(), |p| {
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
    pub async fn reset_profile_sync_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .data()
                .reset_profile_sync_version(id)
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
        data: ProfileUpdateInternal,
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
            cmds.profile().data().profile_last_seen_time(id, last_seen_time)
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
