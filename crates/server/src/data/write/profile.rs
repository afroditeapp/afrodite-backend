use model::{AccountIdInternal, Location, ProfileSearchAgeRangeValidated, ProfileStateInternal, ProfileUpdateInternal, ValidatedSearchGroups};

use crate::{
    data::{
        cache::CacheError, index::location::LocationIndexIteratorState, write::db_transaction,
        DataError, IntoDataError,
    },
    result::{Result, WrappedContextExt},
};

use tracing::info;


define_write_commands!(WriteCommandsProfile);

impl WriteCommandsProfile<'_> {
    pub async fn profile_update_location(
        self,
        id: AccountIdInternal,
        coordinates: Location,
    ) -> Result<(), DataError> {
        let location = self
            .cache()
            .read_cache(id.as_id(), |e| {
                e.profile.as_ref().map(|p| p.location.clone())
            })
            .await
            .into_data_error(id)?
            .ok_or(DataError::FeatureDisabled.report())?;

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
        self.write_cache(id, |entry| {
            let p = entry
                .profile
                .as_mut()
                .ok_or(CacheError::FeatureNotEnabled)?;
            p.location.current_position = new_location_key;
            p.location.current_iterator = new_iterator_state;
            Ok(())
        })
        .await?;

        Ok(())
    }

    pub async fn profile(
        self,
        id: AccountIdInternal,
        data: ProfileUpdateInternal,
    ) -> Result<(), DataError> {
        let profile_data = data.clone();
        let account = db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile(id, &profile_data)?;
            cmds.profile().data().upsert_profile_attributes(id, profile_data.new_data.attributes)?;
            cmds.read().common().account(id)
        })?;

        let (location, profile_data) = self.cache()
            .write_cache(id.as_id(), |e| {
                let p = e.profile.as_mut().ok_or(CacheError::FeatureNotEnabled)?;

                p.data.update_from(&data.new_data);
                p.attributes.update_from(&data.new_data);
                p.data.version_uuid = data.version;

                Ok((
                    p.location.current_position,
                    p.location_index_profile_data(),
                ))
            })
            .await
            .into_data_error(id)?;

        if account.profile_visibility().is_currently_public() {
            self.location()
                .update_profile_data(id.as_id(), profile_data, location)
                .await?;
        }

        Ok(())
    }

    async fn modify_profile_state(
        self,
        id: AccountIdInternal,
        action: impl FnOnce(&mut ProfileStateInternal),
    ) -> Result<(), DataError> {
        let mut s = self.db_read(move |mut cmd| cmd.profile().data().profile_state(id)).await?;
        action(&mut s);
        let s_cloned = s.clone();
        let account = db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile_state(id, s_cloned)?;
            cmds.read().common().account(id)
        })?;

        let (location, profile_data) = self.cache()
            .write_cache(id.as_id(), |e| {
                let p = e.profile.as_mut().ok_or(CacheError::FeatureNotEnabled)?;

                p.state = s.into();

                Ok((
                    p.location.current_position,
                    p.location_index_profile_data(),
                ))
            })
            .await
            .into_data_error(id)?;

        if account.profile_visibility().is_currently_public() {
            self.location()
                .update_profile_data(id.as_id(), profile_data, location)
                .await?;
        }

        Ok(())
    }

    pub async fn profile_name(self, id: AccountIdInternal, data: String) -> Result<(), DataError> {
        let profile_data = data.clone();
        db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile_name(id, profile_data)
        })?;

        self.cache()
            .write_cache(id.as_id(), |e| {
                let p = e.profile.as_mut().ok_or(CacheError::FeatureNotEnabled)?;
                p.data.name = data;
                Ok(())
            })
            .await
            .into_data_error(id)?;

        Ok(())
    }

    pub async fn insert_favorite_profile(
        self,
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
        self,
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
        self,
        sha256: String,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let current_hash = cmds.read().profile().data().attribute_file_hash()?;

            if current_hash.as_deref() != Some(&sha256) {
                info!(
                    "Profile attributes file hash changed from {:?} to {:?}",
                    current_hash, Some(&sha256)
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
            cmds.profile().data().reset_profile_attributes_sync_version(id)
        })
    }

    pub async fn update_search_groups(
        self,
        id: AccountIdInternal,
        search_groups: ValidatedSearchGroups,
    ) -> Result<(), DataError> {
        self.modify_profile_state(id, |s|
            s.search_group_flags = search_groups.into()
        ).await
    }

    pub async fn update_search_age_range(
        self,
        id: AccountIdInternal,
        range: ProfileSearchAgeRangeValidated,
    ) -> Result<(), DataError> {
        self.modify_profile_state(id, |s| {
            s.search_age_range_min = range.min();
            s.search_age_range_max = range.max();
        }).await
    }

    pub async fn benchmark_update_profile_bypassing_cache(
        self,
        id: AccountIdInternal,
        data: ProfileUpdateInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.profile().data().profile(id, &data)
        })
    }
}
