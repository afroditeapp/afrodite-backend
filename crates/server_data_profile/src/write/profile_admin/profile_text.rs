use database_profile::current::read::GetDbReadCommandsProfile;
use server_data::{cache::profile::UpdateLocationCacheState, define_cmd_wrapper_write, read::DbReadCommon, result::WrappedContextExt};

use model_profile::{
    AccountIdInternal, ProfileTextModerationRejectedReasonCategory, ProfileTextModerationRejectedReasonDetails, ProfileVersion
};
use server_data::{
    result::Result,
    DataError, IntoDataError,
};

use crate::{cache::CacheWriteProfile, write::DbTransactionProfile};

define_cmd_wrapper_write!(WriteCommandsProfileAdminProfileText);

impl WriteCommandsProfileAdminProfileText<'_> {
    #[allow(clippy::too_many_arguments)]
    pub async fn moderate_profile_text(
        &self,
        moderator_id: AccountIdInternal,
        name_owner_id: AccountIdInternal,
        text: String,
        accept: bool,
        rejected_category: Option<ProfileTextModerationRejectedReasonCategory>,
        rejected_details: Option<ProfileTextModerationRejectedReasonDetails>,
        move_to_human_moderation: bool,
    ) -> Result<(), DataError> {
        let current_profile = self.db_read(move |mut cmds| cmds.profile().data().profile(name_owner_id)).await?;
        let current_profile_state = self.db_read(move |mut cmds| cmds.profile().data().profile_state(name_owner_id)).await?;
        if current_profile.ptext != text {
            return Err(DataError::NotAllowed.report());
        }
        if current_profile_state.profile_text_moderation_state.is_moderated() {
            return Err(DataError::NotAllowed.report());
        }

        // Profile text accepted value is part of Profile, so update it's version
        let new_profile_version = ProfileVersion::new_random();
        let new_state = db_transaction!(self, move |mut cmds| {
            cmds.profile().data().only_profile_version(name_owner_id, new_profile_version)?;
            cmds.profile().data().increment_profile_sync_version(name_owner_id)?;
            let new_state = if move_to_human_moderation {
                cmds.profile_admin().profile_text().move_to_human_moderation(
                    name_owner_id,
                )?
            } else {
                cmds.profile_admin().profile_text().moderate_profile_text(
                    moderator_id,
                    name_owner_id,
                    accept,
                    rejected_category,
                    rejected_details,
                )?
            };
            Ok(new_state)
        })?;

        self
            .write_cache_profile(name_owner_id.as_id(), |p| {
                p.state.profile_text_moderation_state = new_state;
                p.data.version_uuid = new_profile_version;
                Ok(())
            })
            .await
            .into_data_error(name_owner_id)?;

        self.update_location_cache_profile(name_owner_id).await?;

        Ok(())
    }
}
