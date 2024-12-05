use database_media::current::{read::GetDbReadCommandsMedia, write::GetDbWriteCommandsMedia};
use model::{ContentIdInternal, AccountIdInternal, ProfileContentVersion};
use model_media::{ProfileContentModerationRejectedReasonCategory, ProfileContentModerationRejectedReasonDetails};
use server_data::{cache::profile::UpdateLocationCacheState, define_cmd_wrapper_write, read::DbRead, result::WrappedContextExt, write::DbTransaction, DataError, IntoDataError};

use server_common::result::Result;

use crate::{cache::CacheWriteMedia, write::{media::InitialContentModerationResult, GetWriteCommandsMedia}};

pub struct ModerationResult {
    pub moderation_result: InitialContentModerationResult,
}

define_cmd_wrapper_write!(WriteCommandsProfileAdminContent);

impl WriteCommandsProfileAdminContent<'_> {
    #[allow(clippy::too_many_arguments)]
    pub async fn moderate_profile_content(
        &self,
        moderator_id: AccountIdInternal,
        content_id: ContentIdInternal,
        accept: bool,
        rejected_category: Option<ProfileContentModerationRejectedReasonCategory>,
        rejected_details: Option<ProfileContentModerationRejectedReasonDetails>,
        move_to_human_moderation: bool,
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

        // Profile content accepted value is part of profile content, so update it's version
        let new_profile_content_version = ProfileContentVersion::new_random();
        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .only_profile_content_version(content_id.content_owner(), new_profile_content_version)?;
            cmds.media()
                .media_content()
                .increment_media_content_sync_version(content_id.content_owner())?;
            if move_to_human_moderation {
                cmds.media_admin()
                    .media_content()
                    .move_to_human_moderation(content_id)?;
            } else {
                cmds.media_admin().media_content().moderate_profile_content(
                    moderator_id,
                    content_id,
                    accept,
                    rejected_category,
                    rejected_details,
                )?;
            }
            Ok(())
        })?;

        self.write_cache_media(content_id.content_owner(), |m| {
            m.profile_content_version = new_profile_content_version;
            Ok(())
        })
        .await
        .into_data_error(content_id.content_owner())?;

        self.update_location_cache_profile(content_id.content_owner()).await?;

        let visibility_change = self.handle().media().remove_pending_state_from_profile_visibility_if_needed(content_id.content_owner()).await?;

        Ok(ModerationResult {
            moderation_result: visibility_change,
        })
    }
}
