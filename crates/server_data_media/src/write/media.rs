use std::collections::HashSet;

use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use database_media::current::{read::GetDbReadCommandsMedia, write::GetDbWriteCommandsMedia};
use error_stack::ResultExt;
use model::{Account, ProfileVisibility};
use model_media::{
    AccountIdInternal, ContentId, ContentIdDb, ContentSlot, CurrentAccountMediaInternal, NewContentParams, ProfileContentVersion, SetProfileContent
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
        let content_before_update = self.db_read(move |mut cmds| cmds.media().media_content().current_account_media(id)).await?;

        let new_profile_content_version = ProfileContentVersion::new_random();

        db_transaction!(self, move |mut cmds| {
            cmds.media().media_content().update_profile_content(
                id,
                new,
                new_profile_content_version,
            )?;
            cmds.media().media_content().increment_media_content_sync_version(id)
        })?;

        self.write_cache_media(id.as_id(), |e| {
            e.profile_content_version = new_profile_content_version;
            Ok(())
        })
        .await?;

        self.update_location_cache_profile(id).await?;

        self.update_content_usage(id, content_before_update).await?;

        self.remove_pending_state_from_profile_visibility_if_needed(id).await
    }

    pub async fn update_security_content(
        &self,
        content_owner: AccountIdInternal,
        content: ContentId,
    ) -> Result<(), DataError> {
        let content_before_update = self.db_read(move |mut cmds| cmds.media().media_content().current_account_media(content_owner)).await?;

        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .update_security_content(content_owner, content)?;

            cmds.media().media_content().increment_media_content_sync_version(content_owner)
        })?;

        self.update_content_usage(content_owner, content_before_update).await
    }

    // TODO(prod): Admin is removing content from server. Should only image data
    // be replaced or image data and content ID be removed?

    /// The content must not be in use.
    pub async fn delete_content(
        &self,
        account_id: AccountIdInternal,
        content: ContentId,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media()
                .media_content()
                .delete_content(content)
        })?;

        self.files().media_content(account_id.uuid, content).remove_if_exists().await?;

        Ok(())
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

    pub async fn update_content_usage(
        &self,
        content_owner: AccountIdInternal,
        previous: CurrentAccountMediaInternal,
    ) -> Result<(), DataError> {

        db_transaction!(self, move |mut cmds| {
            let current = cmds.read().media().media_content().current_account_media(content_owner)?;
            let current = HashSet::<ContentIdDb>::from_iter(current.iter_all_content().map(|v| v.id));
            let previous = HashSet::<ContentIdDb>::from_iter(previous.iter_all_content().map(|v| v.id));

            for removed in previous.difference(&current) {
                cmds.media().media_content().change_usage_to_ended(*removed)?;
            }

            for added in current.difference(&previous) {
                cmds.media().media_content().change_usage_to_started(*added)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    pub async fn reset_media_content_sync_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.media().reset_media_content_sync_version(id)
        })
    }
}
