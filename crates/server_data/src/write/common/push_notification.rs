use database::current::write::GetDbWriteCommandsCommon;
use model::{
    AccountIdInternal, PendingNotification, PendingNotificationFlags, PendingNotificationToken,
    PushNotificationDeviceToken,
};
use server_common::data::IntoDataError;

use crate::{
    DataError,
    cache::{CacheReadCommon, CacheWriteCommon},
    db_transaction, define_cmd_wrapper_write,
    result::Result,
    write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsCommonPushNotification);

impl WriteCommandsCommonPushNotification<'_> {
    pub async fn remove_push_notification_device_token_and_pending_notification_token(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .push_notification()
                .remove_push_notification_device_token_and_pending_notification_token(id)
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
    ) -> Result<PendingNotificationToken, DataError> {
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

    pub async fn get_and_reset_pending_notification_with_notification_token(
        &self,
        token: PendingNotificationToken,
    ) -> Result<(AccountIdInternal, PendingNotification), DataError> {
        let (id, flags) = db_transaction!(self, move |mut cmds| {
            cmds.common()
                .push_notification()
                .get_and_reset_pending_notification_with_notification_token(token)
        })?;

        self.write_cache_common(id, |entry| {
            entry.pending_notification_flags = PendingNotificationFlags::empty();
            Ok(())
        })
        .await
        .into_error()?;

        Ok((id, flags))
    }

    pub async fn save_current_notification_flags_to_database_if_needed(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        let flags = self
            .read_cache_common(id, move |entry| Ok(entry.pending_notification_flags))
            .await?;

        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .push_notification()
                .save_current_notification_flags_to_database_if_needed(id, flags.into())
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
}
