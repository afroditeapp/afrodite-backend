use database::current::read::GetDbReadCommandsCommon;
use database_media::current::{read::GetDbReadCommandsMedia, write::GetDbWriteCommandsMedia};
use model::{AccountIdInternal, ContentIdInternal, EventToClientInternal};
use model_media::{
    MediaContentModerationRejectedReasonCategory, MediaContentModerationRejectedReasonDetails,
    ProfileContentModificationMetadata,
};
use server_common::result::Result;
use server_data::{
    DataError, db_manager::InternalWriting, db_transaction, define_cmd_wrapper_write, read::DbRead,
    result::WrappedContextExt, write::DbTransaction,
};

use crate::write::GetWriteCommandsMedia;

define_cmd_wrapper_write!(WriteCommandsProfileAdminContent);

impl WriteCommandsProfileAdminContent<'_> {
    pub async fn moderate_media_content(
        &self,
        mode: ContentModerationMode,
        content_id: ContentIdInternal,
    ) -> Result<(), DataError> {
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
                ContentModerationMode::MoveToHumanModeration {
                    rejected_category,
                    rejected_details,
                } => {
                    cmds.media_admin()
                        .media_content()
                        .move_to_human_moderation(
                            content_id,
                            rejected_category,
                            rejected_details,
                        )?;
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

        self.handle()
            .media()
            .remove_pending_state_from_profile_visibility_if_needed(content_id.content_owner())
            .await?;

        Ok(())
    }

    pub async fn change_face_detected_value(
        &self,
        content_id: ContentIdInternal,
        value: Option<bool>,
    ) -> Result<bool, DataError> {
        let current_content = self
            .db_read(move |mut cmds| {
                cmds.media()
                    .media_content()
                    .get_media_content_raw(content_id)
            })
            .await?;
        if current_content.manual_face_detected() == value {
            // Already done
            return Ok(false);
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

        self.events()
            .send_connected_event(
                content_id.content_owner(),
                EventToClientInternal::MediaContentChanged,
            )
            .await?;

        Ok(true)
    }

    pub async fn change_face_verified_values(
        &self,
        moderator_id: AccountIdInternal,
        values: Vec<(ContentIdInternal, Option<bool>)>,
    ) -> Result<(), DataError> {
        if values.is_empty() {
            return Ok(());
        }

        let content_owner = values[0].0.content_owner();
        if values
            .iter()
            .any(|(content_id, _)| content_id.content_owner() != content_owner)
        {
            return Err(DataError::NotAllowed.report());
        }

        let cache_update = db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .increment_media_content_sync_version(content_owner)?;

            let moderator_is_bot = cmds
                .read()
                .common()
                .state()
                .other_shared_state(moderator_id)?
                .is_bot();

            for (content_id, value) in values.iter().cloned() {
                if moderator_is_bot {
                    cmds.media_admin()
                        .media_content()
                        .change_face_verified_value(content_id, value)?;
                } else {
                    cmds.media_admin()
                        .media_content()
                        .change_face_verified_manual_value(content_id, value)?;
                }
            }

            let current_account_media = cmds
                .read()
                .media()
                .media_content()
                .current_account_media(content_owner)?;

            for (content_id, _) in values {
                if current_account_media
                    .iter_current_profile_content()
                    .any(|v| v.content_id() == content_id.content_id())
                {
                    // Public face verified value might have changed
                    let modification = ProfileContentModificationMetadata::generate();
                    cmds.media()
                        .media_content()
                        .required_changes_for_public_profile_content_update(
                            content_id.content_owner(),
                            &modification,
                        )?;
                    return Ok(Some(modification));
                }
            }

            Ok(None)
        })?;

        if let Some(modification) = cache_update {
            self.handle()
                .media()
                .public_profile_content_cache_update(content_owner, &modification)
                .await?;
        }

        self.events()
            .send_connected_event(content_owner, EventToClientInternal::MediaContentChanged)
            .await?;

        Ok(())
    }

    pub async fn change_security_content_verified_value(
        &self,
        moderator_id: AccountIdInternal,
        account: AccountIdInternal,
        value: Option<bool>,
    ) -> Result<(), DataError> {
        let current_account_media = self
            .db_read(move |mut cmds| cmds.media().media_content().current_account_media(account))
            .await?;

        let moderator_is_bot = self
            .db_read(move |mut cmds| {
                cmds.common()
                    .state()
                    .other_shared_state(moderator_id)
                    .map(|v| v.is_bot())
            })
            .await?;

        let value_already_set = if moderator_is_bot {
            current_account_media.security_content_verified == value
        } else {
            current_account_media.security_content_verified_manual == value
        };
        if value_already_set {
            return Ok(());
        }

        let modification = db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .increment_media_content_sync_version(account)?;

            if moderator_is_bot {
                cmds.media_admin()
                    .media_content()
                    .change_security_content_verified_value(account, value)?;
            } else {
                cmds.media_admin()
                    .media_content()
                    .change_security_content_verified_manual_value(account, value)?;
            }

            // Security content verified value is public info, so update
            // public profile content version and edit time.
            let modification = ProfileContentModificationMetadata::generate();
            cmds.media()
                .media_content()
                .required_changes_for_public_profile_content_update(account, &modification)?;

            Ok(modification)
        })?;

        self.handle()
            .media()
            .public_profile_content_cache_update(account, &modification)
            .await?;

        self.events()
            .send_connected_event(account, EventToClientInternal::MediaContentChanged)
            .await?;

        Ok(())
    }
}

pub enum ContentModerationMode {
    MoveToHumanModeration {
        rejected_category: Option<MediaContentModerationRejectedReasonCategory>,
        rejected_details: Option<MediaContentModerationRejectedReasonDetails>,
    },
    Moderate {
        moderator_id: AccountIdInternal,
        accept: bool,
        rejected_category: Option<MediaContentModerationRejectedReasonCategory>,
        rejected_details: Option<MediaContentModerationRejectedReasonDetails>,
    },
}
