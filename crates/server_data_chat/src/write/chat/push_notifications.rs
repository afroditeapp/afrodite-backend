use model::{AccountIdInternal, FcmDeviceToken, PendingNotification, PushNotificationStateInfo};
use server_data::define_server_data_write_commands;

use server_data::write::WriteCommandsProvider;
use server_data::{result::Result, DataError};

define_server_data_write_commands!(WriteCommandsChatPushNotifications);
define_db_transaction_command!(WriteCommandsChatPushNotifications);

impl <C: WriteCommandsProvider> WriteCommandsChatPushNotifications<C> {
    pub async fn remove_device_token(&mut self, id: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .update_fcm_device_token(id, None)
        })?;

        Ok(())
    }

    pub async fn set_device_token(
        &mut self,
        id: AccountIdInternal,
        token: FcmDeviceToken,
    ) -> Result<(), DataError> {
        let token_clone = token.clone();
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .update_fcm_device_token(id, Some(token_clone))
        })?;

        Ok(())
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

    pub async fn get_and_reset_pending_notification_with_device_token(
        &mut self,
        token: FcmDeviceToken,
    ) -> Result<PendingNotification, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .push_notifications()
                .get_and_reset_pending_notification_with_device_token(token)
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
}
