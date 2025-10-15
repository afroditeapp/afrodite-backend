use database_media::current::{read::GetDbReadCommandsMedia, write::GetDbWriteCommandsMedia};
use model::{AccountIdInternal, ContentIdInternal};
use model_media::{
    MediaContentModerationRejectedReasonCategory, MediaContentModerationRejectedReasonDetails,
    ProfileContentModificationMetadata,
};
use server_common::result::Result;
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, read::DbRead, result::WrappedContextExt,
    write::DbTransaction,
};

use crate::write::{GetWriteCommandsMedia, media::InitialContentModerationResult};

pub struct ModerationResult {
    pub moderation_result: InitialContentModerationResult,
}

define_cmd_wrapper_write!(WriteCommandsProfileAdminContent);

impl WriteCommandsProfileAdminContent<'_> {
    pub async fn moderate_media_content(
        &self,
        mode: ContentModerationMode,
        content_id: ContentIdInternal,
    ) -> Result<ModerationResult, DataError> {
        let current_content = self
            .db_read(move |mut cmds| {
                cmds.media()
                    .media_content()
                    .get_media_content_raw(content_id)
            })
            .await?;
        if current_content.state().is_in_slot() {
            return Err(DataError::NotAllowed.report());
        }

        let cache_update = db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .increment_media_content_sync_version(content_id.content_owner())?;

            match mode {
                ContentModerationMode::MoveToHumanModeration => {
                    cmds.media_admin()
                        .media_content()
                        .move_to_human_moderation(content_id)?;
                }
                ContentModerationMode::Moderate {
                    moderator_id,
                    accept,
                    rejected_category,
                    rejected_details,
                } => {
                    cmds.media_admin().media_content().moderate_media_content(
                        moderator_id,
                        content_id,
                        accept,
                        rejected_category,
                        rejected_details,
                    )?;
                }
            };

            let current_account_media = cmds
                .read()
                .media()
                .media_content()
                .current_account_media(content_id.content_owner())?;
            if current_account_media
                .iter_current_profile_content()
                .any(|v| v.content_id() == content_id.content_id())
            {
                // Public profile content accepted value might have
                // changed, so update public profile content version
                // and edit time.
                let modification = ProfileContentModificationMetadata::generate();
                cmds.media()
                    .media_content()
                    .required_changes_for_public_profile_content_update(
                        content_id.content_owner(),
                        &modification,
                    )?;
                Ok(Some(modification))
            } else {
                Ok(None)
            }
        })?;

        if let Some(modification) = cache_update {
            self.handle()
                .media()
                .public_profile_content_cache_update(content_id.content_owner(), &modification)
                .await?;
        }

        let visibility_change = self
            .handle()
            .media()
            .remove_pending_state_from_profile_visibility_if_needed(content_id.content_owner())
            .await?;

        Ok(ModerationResult {
            moderation_result: visibility_change,
        })
    }

    pub async fn change_face_detected_value(
        &self,
        content_id: ContentIdInternal,
        value: bool,
    ) -> Result<(), DataError> {
        let current_content = self
            .db_read(move |mut cmds| {
                cmds.media()
                    .media_content()
                    .get_media_content_raw(content_id)
            })
            .await?;
        if current_content.face_detected == value {
            // Already done
            return Ok(());
        }

        let cache_update = db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .increment_media_content_sync_version(content_id.content_owner())?;

            cmds.media_admin()
                .media_content()
                .change_face_detected_value(content_id, value)?;

            let current_account_media = cmds
                .read()
                .media()
                .media_content()
                .current_account_media(content_id.content_owner())?;
            if current_account_media
                .iter_current_profile_content()
                .any(|v| v.content_id() == content_id.content_id())
            {
                // Public profile content [model_media::ContentInfo::p] value might have
                // changed, so update public profile content version
                // and edit time.
                let modification = ProfileContentModificationMetadata::generate();
                cmds.media()
                    .media_content()
                    .required_changes_for_public_profile_content_update(
                        content_id.content_owner(),
                        &modification,
                    )?;
                Ok(Some(modification))
            } else {
                Ok(None)
            }
        })?;

        if let Some(modification) = cache_update {
            self.handle()
                .media()
                .public_profile_content_cache_update(content_id.content_owner(), &modification)
                .await?;
        }

        Ok(())
    }
}

pub enum ContentModerationMode {
    MoveToHumanModeration,
    Moderate {
        moderator_id: AccountIdInternal,
        accept: bool,
        rejected_category: Option<MediaContentModerationRejectedReasonCategory>,
        rejected_details: Option<MediaContentModerationRejectedReasonDetails>,
    },
}
