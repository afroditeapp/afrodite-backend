use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, Location, ProfileLink, ProfileUpdateInternal};

use crate::data::{
    cache::CacheError, index::location::LocationIndexIteratorState, DataError, IntoDataError,
};

define_write_commands!(WriteCommandsProfile);

impl WriteCommandsProfile<'_> {
    pub async fn profile_update_visibility(
        self,
        id: AccountIdInternal,
        public: bool,
        update_only_if_no_value: bool,
    ) -> Result<(), DataError> {
        let (visiblity, location, profile_link) = self
            .cache()
            .write_cache(id.as_id(), |e| {
                let p = e.profile.as_mut().ok_or(CacheError::FeatureNotEnabled)?;

                // Handle race condition between remote fetch and update.
                // Update will override the initial fetch.
                if update_only_if_no_value {
                    if p.public.is_none() {
                        p.public = Some(public);
                    }
                } else {
                    p.public = Some(public);
                }

                Ok((
                    p.public.unwrap_or_default(),
                    p.location.current_position,
                    ProfileLink::new(id.as_id(), &p.data),
                ))
            })
            .await
            .into_data_error(id)?;

        if visiblity {
            self.location()
                .update_profile_link(id.as_id(), profile_link, location)
                .await?;
        } else {
            self.location()
                .remove_profile_link(id.as_id(), location)
                .await?;
        }

        Ok(())
    }

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
            .ok_or(DataError::FeatureDisabled)?;

        let new_location_key = self.location().coordinates_to_key(&coordinates);
        self.db_write(move |cmds| cmds.into_profile().data().profile_location(id, coordinates))
            .await?;

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
        self.db_write(move |cmds| cmds.into_profile().data().profile(id, profile_data))
            .await?;

        self.cache()
            .write_cache(id.as_id(), |e| {
                let p = e.profile.as_mut().ok_or(CacheError::FeatureNotEnabled)?;

                p.data.profile_text = data.new_data.profile_text;
                p.data.version_uuid = data.version;
                Ok(())
            })
            .await
            .into_data_error(id)?;

        Ok(())
    }

    pub async fn profile_name(self, id: AccountIdInternal, data: String) -> Result<(), DataError> {
        let profile_data = data.clone();
        self.db_write(move |cmds| cmds.into_profile().data().profile_name(id, profile_data))
            .await?;

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
        self.db_write(move |cmds| cmds.into_profile().favorite().insert_favorite_profile(id, favorite))
            .await
    }

    pub async fn remove_favorite_profile(
        self,
        id: AccountIdInternal,
        favorite: AccountIdInternal,
    ) -> Result<(), DataError> {
        self.db_write(move |cmds| cmds.into_profile().favorite().remove_favorite_profile(id, favorite))
            .await
    }

    pub async fn benchmark_update_profile_bypassing_cache(
        self,
        id: AccountIdInternal,
        data: ProfileUpdateInternal,
    ) -> Result<(), DataError> {
        self.db_write(move |cmds| cmds.into_profile().data().profile(id, data))
            .await?;

        //self.cmds.update_data(id, &data).await?;

        Ok(())
    }
}
