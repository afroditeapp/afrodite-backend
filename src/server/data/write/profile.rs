use std::net::SocketAddr;

use crate::{api::{model::{AccountIdInternal, AuthPair, ProfileLink, Location}, media::data::{Moderation, HandleModerationRequest, ModerationRequestContent, ContentId, PrimaryImage}}, server::data::{DatabaseError, file::file::ImageSlot, cache::CacheError, database::sqlite::SqliteUpdateJson}, utils::ConvertCommandError};

use error_stack::{Result, ResultExt, Report};



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

        let index = self.location().get().ok_or(DatabaseError::FeatureDisabled)?;
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

        let write_to_index = self.location().get().ok_or(DatabaseError::FeatureDisabled)?;
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
}
