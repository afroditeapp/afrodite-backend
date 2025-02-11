use database_media::current::{read::GetDbReadCommandsMedia, write::GetDbWriteCommandsMedia};
use model::{ContentIdInternal, AccountIdInternal, ProfileContentVersion};
use model_media::{ProfileContentEditedTime, ProfileContentModerationRejectedReasonCategory, ProfileContentModerationRejectedReasonDetails};
use server_data::{define_cmd_wrapper_write, read::DbRead, result::WrappedContextExt, write::DbTransaction, DataError};

use server_common::result::Result;

use crate::write::{media::InitialContentModerationResult, GetWriteCommandsMedia};

pub struct ModerationResult {
    pub moderation_result: InitialContentModerationResult,
}

define_cmd_wrapper_write!(WriteCommandsProfileAdminContent);

impl WriteCommandsProfileAdminContent<'_> {
    #[allow(clippy::too_many_arguments)]
    pub async fn moderate_profile_content(
        &self,
        mode: ContentModerationMode,
        content_id: ContentIdInternal,
    ) -> Result<ModerationResult, DataError> {
        let current_content = self
            .db_read(move |mut cmds| cmds.media().media_content().get_media_content_raw(content_id))
            .await?;
        if current_content
            .state()
            .is_in_slot()
        {
            return Err(DataError::NotAllowed.report());
        }

        let cache_update = db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .increment_media_content_sync_version(content_id.content_owner())?;
            let cache_update = match mode {
                ContentModerationMode::MoveToHumanModeration => {
                    cmds.media_admin()
                        .media_content()
                        .move_to_human_moderation(content_id)?;
                    None
                }
                ContentModerationMode::Moderate {
                    moderator_id,
                    accept,
                    rejected_category,
                    rejected_details,
                } => {
                    cmds.media_admin().media_content().moderate_profile_content(
                        moderator_id,
                        content_id,
                        accept,
                        rejected_category,
                        rejected_details,
                    )?;

                    let current_account_media = cmds.read().media().media_content().current_account_media(content_id.content_owner())?;
                    if current_account_media.iter_current_profile_content().any(|v| v.content_id() == content_id.content_id()) {
                        // Public profile content accepted value might have
                        // changed, so update public profile content version
                        // and edit time.
                        let version = ProfileContentVersion::new_random();
                        let edit_time = ProfileContentEditedTime::current_time();
                        cmds.media()
                            .media_content()
                            .required_changes_for_public_profile_content_update(content_id.content_owner(), version, edit_time)?;
                        Some((version, edit_time))
                    } else {
                        None
                    }
                }
            };
            Ok(cache_update)
        })?;

        if let Some(update) = cache_update {
            self.handle().media().public_profile_content_cache_update(content_id.content_owner(), update).await?;
        }

        let visibility_change = self.handle().media().remove_pending_state_from_profile_visibility_if_needed(content_id.content_owner()).await?;

        Ok(ModerationResult {
            moderation_result: visibility_change,
        })
    }
}

pub enum ContentModerationMode {
    MoveToHumanModeration,
    Moderate {
        moderator_id: AccountIdInternal,
        accept: bool,
        rejected_category: Option<ProfileContentModerationRejectedReasonCategory>,
        rejected_details: Option<ProfileContentModerationRejectedReasonDetails>,
    }
}
