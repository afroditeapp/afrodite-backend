use model::{AccountIdInternal, FcmDeviceToken, PendingNotification, PendingNotificationToken, PushNotificationStateInfo};
use server_data::{
    cache::CacheReadCommon, define_cmd_wrapper, result::Result, DataError
};

use crate::write::DbTransactionChat;

define_cmd_wrapper!(WriteCommandsChatPushNotifications);

impl<C: DbTransactionChat + CacheReadCommon> WriteCommandsChatPushNotifications<C> {
    pub async fn remove_fcm_device_token_and_pending_notification_token(&mut self, id: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .remove_fcm_device_token_and_pending_notification_token(id)
        })?;

        Ok(())
    }

    pub async fn remove_fcm_device_token(&mut self, id: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .remove_fcm_device_token(id)
        })?;

        Ok(())
    }

    pub async fn set_device_token(
        &mut self,
        id: AccountIdInternal,
        token: FcmDeviceToken,
    ) -> Result<PendingNotificationToken, DataError> {
        let token_clone = token.clone();
        let token = db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .update_fcm_device_token_and_generate_new_notification_token(id, token_clone)
        })?;

        Ok(token)
    }

    pub async fn reset_pending_notification(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .reset_pending_notification(id)
        })
    }

    pub async fn get_and_reset_pending_notification_with_notification_token(
        &mut self,
        token: PendingNotificationToken,
    ) -> Result<(AccountIdInternal, PendingNotification), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .get_and_reset_pending_notification_with_notification_token(token)
        })
    }

    pub async fn enable_push_notification_sent_flag(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .enable_push_notification_sent_flag(id)
        })
    }

    pub async fn get_push_notification_state_info_and_add_notification_value(
        &mut self,
        id: AccountIdInternal,
        notification: PendingNotification,
    ) -> Result<PushNotificationStateInfo, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .get_push_notification_state_info_and_add_notification_value(id, notification)
        })
    }

    pub async fn save_current_non_empty_notification_flags_from_cache_to_database(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        let flags = self.read_cache_common(id, move |entry| {
            Ok(entry.pending_notification_flags)
        })
        .await?;

        if flags.is_empty() {
            return Ok(());
        }

        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .get_push_notification_state_info_and_add_notification_value(id, flags.into())
        })
        .map(|_| ())
    }
}
