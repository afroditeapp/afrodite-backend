use database_profile::current::{read::GetDbReadCommandsProfile, write::GetDbWriteCommandsProfile};
use model_profile::{
    AccountIdInternal, ProfileModerationRejectedReasonCategory,
    ProfileModerationRejectedReasonDetails, ProfileModificationMetadata,
    ProfileStringModerationContentType, ProfileStringModerationState,
};
use server_data::{
    DataError, IntoDataError,
    cache::profile::UpdateLocationCacheState,
    db_transaction, define_cmd_wrapper_write,
    read::DbRead,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
};

use crate::cache::CacheWriteProfile;

define_cmd_wrapper_write!(WriteCommandsProfileAdminModeration);

impl WriteCommandsProfileAdminModeration<'_> {
    pub async fn moderate_profile_string(
        &self,
        content_type: ProfileStringModerationContentType,
        mode: ModerateProfileValueMode,
        string_owner_id: AccountIdInternal,
        string_value: String,
    ) -> Result<(), DataError> {
        let current_profile = self
            .db_read(move |mut cmds| cmds.profile().data().profile(string_owner_id))
            .await?;
        let current_value = match content_type {
            ProfileStringModerationContentType::ProfileName => current_profile.name,
            ProfileStringModerationContentType::ProfileText => current_profile.ptext,
        };
        if current_value != string_value {
            return Err(DataError::NotAllowed.report());
        }

        let current_moderation_state = self
            .db_read(move |mut cmds| {
                cmds.profile()
                    .moderation()
                    .profile_moderation_info(string_owner_id, content_type)
            })
            .await?;
        if current_moderation_state.is_none() {
            return Err(DataError::NotAllowed.report());
        }

        // Profile name and text have accepted boolean in Profile, so update Profile metadata
        let modification = ProfileModificationMetadata::generate();
        let new_state: ProfileStringModerationState = db_transaction!(self, move |mut cmds| {
            cmds.profile()
                .data()
                .required_changes_for_profile_update(string_owner_id, &modification)?;
            let new_state = match mode {
                ModerateProfileValueMode::MoveToHumanModeration => cmds
                    .profile_admin()
                    .moderation()
                    .move_to_human_moderation(string_owner_id, content_type)?,
                ModerateProfileValueMode::Moderate {
                    moderator_id,
                    accept,
                    rejected_category,
                    rejected_details,
                } => {
                    if content_type == ProfileStringModerationContentType::ProfileName && accept {
                        cmds.profile_admin()
                            .moderation()
                            .add_to_profile_name_allowlist(
                                moderator_id,
                                string_owner_id,
                                string_value,
                            )?;
                    }
                    cmds.profile_admin().moderation().moderate_profile_string(
                        moderator_id,
                        string_owner_id,
                        content_type,
                        accept,
                        rejected_category,
                        rejected_details,
                    )?
                }
            };
            Ok(new_state)
        })?;

        self.write_cache_profile(string_owner_id.as_id(), |p| {
            match content_type {
                ProfileStringModerationContentType::ProfileName => {
                    p.update_profile_name_moderation_state(Some(new_state.into()))
                }
                ProfileStringModerationContentType::ProfileText => {
                    p.update_profile_text_moderation_state(Some(new_state.into()))
                }
            };
            p.update_profile_version_uuid(modification.version);
            p.state.profile_edited_time = modification.time;
            Ok(())
        })
        .await
        .into_data_error(string_owner_id)?;

        self.update_location_cache_profile(string_owner_id).await?;

        Ok(())
    }
}

pub enum ModerateProfileValueMode {
    MoveToHumanModeration,
    Moderate {
        moderator_id: AccountIdInternal,
        accept: bool,
        rejected_category: Option<ProfileModerationRejectedReasonCategory>,
        rejected_details: ProfileModerationRejectedReasonDetails,
    },
}
