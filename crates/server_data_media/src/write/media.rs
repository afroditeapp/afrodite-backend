use error_stack::ResultExt;
use model_media::{
    AccountIdInternal, ContentId, ContentSlot, ModerationRequestContent, ModerationRequestState, NewContentParams, NextQueueNumberType, ProfileContentVersion, ProfileVisibility, SetProfileContent
};
use server_data::{
    cache::profile::UpdateLocationCacheState, define_cmd_wrapper, file::FileWrite, result::{Result, WrappedContextExt}, DataError, DieselDatabaseError
};

use crate::{cache::CacheWriteMedia, read::DbReadMedia};

use super::DbTransactionMedia;

define_cmd_wrapper!(WriteCommandsMedia);

impl<C: DbTransactionMedia + DbReadMedia + FileWrite + CacheWriteMedia + UpdateLocationCacheState> WriteCommandsMedia<C> {
    pub async fn create_or_update_moderation_request(
        &self,
        account_id: AccountIdInternal,
        request: ModerationRequestContent,
    ) -> Result<(), DataError> {
        let current_request = self
            .db_read(move |mut cmds| {
                cmds.media()
                    .moderation_request()
                    .moderation_request(account_id)
            })
            .await?;

        let account = self
            .db_read(move |mut cmds| cmds.common().account(account_id))
            .await?;

        let queue_num_type = match account.profile_visibility() {
            ProfileVisibility::PendingPrivate | ProfileVisibility::PendingPublic => {
                NextQueueNumberType::InitialMediaModeration
            }
            ProfileVisibility::Private | ProfileVisibility::Public => {
                NextQueueNumberType::MediaModeration
            }
        };

        if let Some(current_request) = current_request {
            match current_request.state {
                ModerationRequestState::Waiting => {
                    db_transaction!(self, move |mut cmds| {
                        cmds.media()
                            .moderation_request()
                            .update_moderation_request(account_id, request)
                    })
                }
                ModerationRequestState::InProgress => Err(DataError::NotAllowed.report()),
                ModerationRequestState::Accepted | ModerationRequestState::Rejected => {
                    db_transaction!(self, move |mut cmds| {
                        cmds.media()
                            .moderation_request()
                            .create_new_moderation_request(account_id, request, queue_num_type)
                    })
                }
            }
        } else {
            db_transaction!(self, move |mut cmds| {
                cmds.media()
                    .moderation_request()
                    .create_new_moderation_request(account_id, request, queue_num_type)
            })
        }
    }

    /// Completes previous save_to_tmp.
    pub async fn save_to_slot(
        &self,
        id: AccountIdInternal,
        content_id: ContentId,
        slot: ContentSlot,
        new_content_params: NewContentParams,
        face_detected: bool,
    ) -> Result<(), DataError> {
        // Remove previous slot content.
        let current_content_in_slot = self
            .db_read(move |mut cmds| {
                cmds.media()
                    .moderation_request()
                    .get_media_content_from_slot(id, slot)
            })
            .await?;

        if let Some(content) = current_content_in_slot {
            let path = self.files().media_content(id.as_id(), content.into());
            path.remove_if_exists()
                .await
                .change_context(DataError::File)?;
            self.db_transaction(move |mut cmds| {
                cmds.media()
                    .moderation_request()
                    .delete_content_from_slot(id, slot)
            })
            .await
            .change_context(DataError::Sqlite)?;
        }

        // Paths related to moving content from tmp dir to content dir
        let tmp_img = self
            .files()
            .processed_content_upload(id.as_id(), content_id);
        let processed_content_path = self.files().media_content(id.as_id(), content_id);

        self.db_transaction(move |mut cmds| {
            cmds.media()
                .moderation_request()
                .insert_content_id_to_slot(id, content_id, slot, new_content_params, face_detected)?;

            // Move content from tmp dir to content dir
            tmp_img
                .move_to_blocking(&processed_content_path)
                .map_err(|e| e.change_context(DieselDatabaseError::File))?;
            // If moving fails, diesel rollbacks the transaction.

            Ok(())
        })
        .await?;

        // TODO(prod): Update media backup code
        // self.media_backup()
        //     .backup_jpeg_image(id.as_id(), content_id)
        //     .await
        //     .change_context(DataError::MediaBackup)

        Ok(())
    }

    pub async fn update_profile_content(
        self,
        id: AccountIdInternal,
        new: SetProfileContent,
    ) -> Result<(), DataError> {
        let new_profile_content_version = ProfileContentVersion::new_random();

        db_transaction!(self, move |mut cmds| {
            cmds.media().media_content().update_profile_content(id, new, new_profile_content_version)
        })?;

        self
            .write_cache_media(id.as_id(), |e| {
                e.profile_content_version = new_profile_content_version;
                Ok(())
            })
            .await?;

        self.update_location_cache_profile(id).await?;

        Ok(())
    }

    pub async fn update_or_delete_pending_profile_content(
        self,
        id: AccountIdInternal,
        new: Option<SetProfileContent>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .update_or_delete_pending_profile_content(id, new)
        })
    }

    pub async fn update_security_content(
        self,
        content_owner: AccountIdInternal,
        content: ContentId,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .update_security_content(content_owner, content)
        })
    }

    pub async fn update_or_delete_pending_security_content(
        self,
        content_owner: AccountIdInternal,
        content: Option<ContentId>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .update_or_delete_pending_security_content(content_owner, content)
        })
    }

    pub async fn delete_content(
        self,
        content_owner: AccountIdInternal,
        content: ContentId,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .delete_content(content_owner, content)
        })
    }

    pub async fn delete_moderation_request_not_yet_in_moderation(
        self,
        moderation_request_owner: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .moderation_request()
                .delete_moderation_request_not_yet_in_moderation(moderation_request_owner)
        })
    }
}
