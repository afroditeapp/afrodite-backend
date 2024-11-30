use database_chat::current::write::GetDbWriteCommandsChat;
use model::{AccountIdInternal, FcmDeviceToken, PendingNotification, PendingNotificationToken, PushNotificationStateInfo};
use server_data::{
    cache::CacheReadCommon, define_cmd_wrapper_write, result::Result, DataError, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsChatPushNotifications);

impl WriteCommandsChatPushNotifications<'_> {
    pub async fn remove_fcm_device_token(&self, id: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .remove_fcm_device_token(id)
        })?;

        Ok(())
    }

    pub async fn set_device_token(
        &self,
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
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .reset_pending_notification(id)
        })
    }

    pub async fn get_and_reset_pending_notification_with_notification_token(
        &self,
        token: PendingNotificationToken,
    ) -> Result<(AccountIdInternal, PendingNotification), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .get_and_reset_pending_notification_with_notification_token(token)
        })
    }

    pub async fn enable_push_notification_sent_flag(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .enable_push_notification_sent_flag(id)
        })
    }

    pub async fn get_push_notification_state_info_and_add_notification_value(
        &self,
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
        &self,
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
