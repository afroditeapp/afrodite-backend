
use database::current::write::GetDbWriteCommandsCommon;
use database_profile::current::write::GetDbWriteCommandsProfile;
use model_profile::{
    AccountIdInternal, ProfileVersion,
};
use server_data::{
    app::GetConfig, cache::profile::UpdateLocationCacheState, define_cmd_wrapper_write, result::Result, DataError, IntoDataError, write::DbTransaction
};
use server_data_profile::cache::CacheWriteProfile;

define_cmd_wrapper_write!(UnlimitedLikesUpdate);

impl UnlimitedLikesUpdate<'_> {
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
        db_transaction!(self, move |mut cmds| {
            if is_profile_component_enabled {
                cmds.profile().data().only_profile_version(id, new_profile_version)?;
                cmds.profile().data().increment_profile_sync_version(id)?;
            }
            cmds.common().state().update_unlimited_likes(
                id,
                unlimited_likes_value,
            )
        })?;

        self
            .write_cache_profile_and_common(id.as_id(), |p, e| {
                e.other_shared_state.unlimited_likes = unlimited_likes_value;
                p.data.version_uuid = new_profile_version;
                Ok(())
            })
            .await
            .into_data_error(id)?;

        self.update_location_cache_profile(id).await?;

        Ok(())
    }
}
