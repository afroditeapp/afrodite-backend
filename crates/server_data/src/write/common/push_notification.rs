use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use model::{AccountIdInternal, PushNotificationDeviceToken, PushNotificationEncryptionKey};
use tracing::info;

use crate::{
    DataError, cache::CacheReadCommon, db_transaction, define_cmd_wrapper_write, result::Result,
    write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsCommonPushNotification);

impl WriteCommandsCommonPushNotification<'_> {
    pub async fn remove_push_notification_device_token_and_encryption_key(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .push_notification()
                .remove_push_notification_device_token_and_encryption_key(id)
        })?;

        Ok(())
    }

    pub async fn remove_push_notification_device_token(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .push_notification()
                .remove_push_notification_device_token(id)
        })?;

        Ok(())
    }

    pub async fn set_device_token(
        &self,
        id: AccountIdInternal,
        token: PushNotificationDeviceToken,
    ) -> Result<PushNotificationEncryptionKey, DataError> {
        let token_clone = token.clone();
        let token = db_transaction!(self, move |mut cmds| {
            cmds.common()
                .push_notification()
                .update_push_notification_device_token_and_generate_new_notification_token(
                    id,
                    token_clone,
                )
        })?;

        Ok(token)
    }

    pub async fn save_current_notification_flags_to_database_if_needed(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        let (flags, sent_flags) = self
            .read_cache_common(id, move |entry| {
                Ok((
                    entry.pending_push_notification_flags,
                    entry.sent_push_notification_flags,
                ))
            })
            .await?;

        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .push_notification()
                .save_current_notification_flags_to_database_if_needed(
                    id,
                    flags.into(),
                    sent_flags.into(),
                )
        })
        .map(|_| ())
    }

    pub async fn reset_push_notification_info_sync_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .push_notification()
                .reset_push_notification_info_sync_version(id)
        })?;

        Ok(())
    }

    /// Updates the VAPID public key sha256 and related
    /// sync version for it for every account if needed.
    pub async fn update_vapid_public_key_sha256_and_sync_versions(
        &self,
        sha256: String,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let current_hash = cmds
                .read()
                .common()
                .push_notification()
                .vapid_public_key_hash()?;

            if current_hash.as_deref() != Some(&sha256) {
                info!(
                    "VAPID public key hash changed from {:?} to {:?}",
                    current_hash,
                    Some(&sha256)
                );

                cmds.common()
                    .push_notification()
                    .upsert_vapid_public_key_hash(&sha256)?;

                cmds.common()
                    .push_notification()
                    .increment_push_notification_info_sync_version_for_every_account()?;
            }

            Ok(())
        })
    }
}
