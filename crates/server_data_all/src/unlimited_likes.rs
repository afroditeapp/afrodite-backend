
use model::{
    AccountIdInternal, ProfileVersion,
};
use server_data::{
    define_server_data_write_commands, result::Result, write::WriteCommandsProvider, DataError, IntoDataError
};

define_server_data_write_commands!(UnlimitedLikesUpdate);
define_db_transaction_command!(UnlimitedLikesUpdate);

impl<C: WriteCommandsProvider> UnlimitedLikesUpdate<C> {
    /// Unlimited likes value is needed in both profile and chat component, so
    /// account component owns it. Because of that, update new value code
    /// is located in this crate.
    pub async fn update_unlimited_likes_value(
        &self,
        id: AccountIdInternal,
        unlimited_likes_value: bool,
    ) -> Result<(), DataError> {
        // Unlimited likes value is part of Profile, so update it's version
        // (if profile component is enabled).
        let new_profile_version = ProfileVersion::new_random();
        let is_profile_component_enabled = self.config().components().profile;
        let account = db_transaction!(self, move |mut cmds| {
            if is_profile_component_enabled {
                cmds.profile().data().only_profile_version(id, new_profile_version)?;
                cmds.profile().data().increment_profile_sync_version(id)?;
            }
            cmds.common().state().update_unlimited_likes(
                id,
                unlimited_likes_value,
            )?;
            cmds.read().common().account(id)
        })?;

        let location_and_profile_data = self
            .cache()
            .write_cache(id.as_id(), |e| {
                e.other_shared_state.unlimited_likes = unlimited_likes_value;

                if let Some(p) = e.profile.as_mut() {
                    p.data.version_uuid = new_profile_version;
                    Ok(Some((p.location.current_position, e.location_index_profile_data()?)))
                } else {
                    Ok(None)
                }
            })
            .await
            .into_data_error(id)?;

        if account.profile_visibility().is_currently_public() {
            if let Some((location, profile_data)) = location_and_profile_data {
                self.location()
                .update_profile_data(id.as_id(), profile_data, location)
                .await?;
            }
        }

        Ok(())
    }
}
