use model::{AccountIdInternal, Location, ProfileUpdateInternal};

use crate::{
    data::{
        cache::CacheError, index::location::LocationIndexIteratorState, write::db_transaction,
        DataError, IntoDataError,
    },
    result::{Result, WrappedContextExt},
};

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
