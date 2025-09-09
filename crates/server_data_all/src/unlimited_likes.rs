use database::current::write::GetDbWriteCommandsCommon;
use database_profile::current::write::GetDbWriteCommandsProfile;
use model_chat::ProfileModificationMetadata;
use model_profile::AccountIdInternal;
use server_data::{
    DataError, IntoDataError, cache::profile::UpdateLocationCacheState, db_transaction,
    define_cmd_wrapper_write, result::Result, write::DbTransaction,
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
        let modification = ProfileModificationMetadata::generate();
        db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .data()
                .required_changes_for_profile_update(id, &modification)?;
            cmds.common()
                .state()
                .update_unlimited_likes(id, unlimited_likes_value)
        })?;

        self.write_cache_profile_and_common(id.as_id(), |p, e| {
            e.other_shared_state.unlimited_likes = unlimited_likes_value;
            p.update_profile_version_uuid(modification.version);
            p.state.profile_edited_time = modification.time;
            Ok(())
        })
        .await
        .into_data_error(id)?;

        self.update_location_cache_profile(id).await?;

        Ok(())
    }
}
