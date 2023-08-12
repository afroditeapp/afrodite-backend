use error_stack::{Result, ResultExt};

use database::sqlite::SqliteUpdateJson;
use model::{AccountIdInternal, Location, ProfileLink, ProfileUpdateInternal};

use crate::{
    data::{cache::CacheError, DatabaseError},
    utils::ConvertCommandErrorExt,
};

define_write_commands!(WriteCommandsProfile);

impl WriteCommandsProfile<'_> {
    pub async fn profile_update_visibility(
        self,
        id: AccountIdInternal,
        public: bool,
        update_only_if_no_value: bool,
    ) -> Result<(), DatabaseError> {
        let (visiblity, location, profile_link) = self
            .cache()
            .write_cache(id.as_light(), |e| {
                let p = e
                    .profile
                    .as_mut()
                    .ok_or(CacheError::InitFeatureNotEnabled)?;

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
                    ProfileLink::new(id.as_light(), &p.data),
                ))
            })
            .await
            .convert(id)?;

        let index = self
            .location()
            .get()
            .ok_or(DatabaseError::FeatureDisabled)?;
        if visiblity {
            index
                .update_profile_link(id.as_light(), profile_link, location)
                .await;
        } else {
            index.remove_profile_link(id.as_light(), location).await;
        }

        Ok(())
    }

    pub async fn profile_update_location(
        self,
        id: AccountIdInternal,
        coordinates: Location,
    ) -> Result<(), DatabaseError> {
        let location = self
            .cache()
            .read_cache(id.as_light(), |e| {
                e.profile.as_ref().map(|p| p.location.clone())
            })
            .await
            .convert(id)?
            .ok_or(DatabaseError::FeatureDisabled)?;

        let write_to_index = self
            .location()
            .get()
            .ok_or(DatabaseError::FeatureDisabled)?;
        let new_location_key = write_to_index.coordinates_to_key(coordinates);

        // TODO: Create new database table for location.
        // TODO: Also update history?
        new_location_key
            .update_json(id, &self.current())
            .await
            .change_context(DatabaseError::Sqlite)?;
        write_to_index
            .update_profile_location(id.as_light(), location.current_position, new_location_key)
            .await;

        Ok(())
    }

    pub async fn benchmark_update_profile_bypassing_cache(
        self,
        id: AccountIdInternal,
        data: ProfileUpdateInternal,
    ) -> Result<(), DatabaseError> {
        self.db_write(move |cmds| cmds.profile().update_profile(id, data))
            .await?;

        //self.cmds.update_data(id, &data).await?;

        Ok(())
    }
}
