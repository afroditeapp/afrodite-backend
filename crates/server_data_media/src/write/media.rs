use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use database_media::current::{read::GetDbReadCommandsMedia, write::GetDbWriteCommandsMedia};
use error_stack::ResultExt;
use model::{Account, ProfileVisibility};
use model_media::{
    AccountIdInternal, ContentId, ContentSlot,
    NewContentParams, ProfileContentVersion,
    SetProfileContent,
};
use server_data::{
    app::GetConfig, cache::profile::UpdateLocationCacheState, define_cmd_wrapper_write, file::FileWrite, read::DbRead, result::{Result, WrappedContextExt}, write::{DbTransaction, GetWriteCommandsCommon}, DataError, DieselDatabaseError
};

use crate::cache::CacheWriteMedia;

pub enum InitialContentModerationResult {
    /// Profile visibility changed from pending to normal.
    AllAccepted {
        account_after_visibility_change: Account,
    },
    AllModeratedAndNotAccepted,
    NoChange,
}

define_cmd_wrapper_write!(WriteCommandsMedia);

impl WriteCommandsMedia<'_> {
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
                    .media_content()
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
                    .media_content()
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
                .media_content()
                .insert_content_id_to_slot(
                    id,
                    content_id,
                    slot,
                    new_content_params,
                    face_detected,
                )?;

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
        &self,
        id: AccountIdInternal,
        new: SetProfileContent,
    ) -> Result<InitialContentModerationResult, DataError> {
        let new_profile_content_version = ProfileContentVersion::new_random();

        db_transaction!(self, move |mut cmds| {
            cmds.media().media_content().update_profile_content(
                id,
                new,
                new_profile_content_version,
            )?;
            cmds.media().media_content().increment_profile_content_sync_version(id)
        })?;

        self.write_cache_media(id.as_id(), |e| {
            e.profile_content_version = new_profile_content_version;
            Ok(())
        })
        .await?;

        self.update_location_cache_profile(id).await?;

        self.remove_pending_state_from_profile_visibility_if_needed(id).await
    }

    pub async fn update_security_content(
        &self,
        content_owner: AccountIdInternal,
        content: ContentId,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .update_security_content(content_owner, content)
        })
    }

    pub async fn delete_content(
        &self,
        content_owner: AccountIdInternal,
        content: ContentId,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .delete_content(content_owner, content)
        })
    }

    pub async fn remove_pending_state_from_profile_visibility_if_needed(
        &self,
        content_owner: AccountIdInternal,
    ) -> Result<InitialContentModerationResult, DataError> {
        if !self.config().components().account {
            // TODO(microservice): The media server should request
            // profile visibility change from account server if
            // needed.
            return Err(DataError::FeatureDisabled.report());
        }

        let current_account = self.db_read(move |mut cmds| cmds.common().account(content_owner)).await?;
        let profile_visibility = current_account.profile_visibility();

        let info = db_transaction!(self, move |mut cmds| {
            if !profile_visibility.is_pending() {
                return Ok(InitialContentModerationResult::NoChange)
            }

            let current_media = cmds
                .read()
                .media()
                .media_content()
                .current_account_media(content_owner)?;

            let mut all_accepted = current_media.iter_current_profile_content().count() > 0;
            let mut all_moderated = current_media.iter_current_profile_content().count() > 0;
            for c in current_media.iter_current_profile_content() {
                if !c.state().is_accepted() {
                    all_accepted = false;
                }
                if !c.state().is_moderated() {
                    all_moderated = false;
                }
            }

            if all_accepted {
                let current_account = cmds.read().common().account(content_owner)?;
                let visibility = current_account.profile_visibility();
                let new_visibility = match visibility {
                    ProfileVisibility::Public => ProfileVisibility::Public,
                    ProfileVisibility::Private => ProfileVisibility::Private,
                    ProfileVisibility::PendingPublic => ProfileVisibility::Public,
                    ProfileVisibility::PendingPrivate => ProfileVisibility::Private,
                };
                let new_account = cmds.common().state().update_syncable_account_data(
                    content_owner,
                    current_account.clone(),
                    move |_, _, visibility| {
                        *visibility = new_visibility;
                        Ok(())
                    },
                )?;

                Ok(InitialContentModerationResult::AllAccepted { account_after_visibility_change: new_account})
            } else if all_moderated {
                Ok(InitialContentModerationResult::AllModeratedAndNotAccepted)
            } else {
                Ok(InitialContentModerationResult::NoChange)
            }
        })?;

        if let InitialContentModerationResult::AllAccepted { account_after_visibility_change } = &info {
            self.handle()
                .common()
                .internal_handle_new_account_data_after_db_modification(
                    content_owner,
                    &current_account,
                    account_after_visibility_change,
                )
                .await?;
        }

        Ok(info)
    }
}
