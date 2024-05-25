


use database::current::write::chat::{ChatStateChanges, PushNotificationStateInfo};
use error_stack::ResultExt;
use model::{AccountId, AccountIdInternal, ChatStateRaw, FcmDeviceToken, MessageNumber, PendingMessageId, PendingNotification, SyncVersionUtils};
use simple_backend_database::diesel_db::DieselDatabaseError;
use simple_backend_utils::ContextExt;

use crate::{
    data::{cache::CacheError, write::db_transaction, DataError},
    result::Result,
};

define_write_commands!(WriteCommandsChatPushNotifications);

impl WriteCommandsChatPushNotifications<'_> {
    pub async fn remove_device_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().push_notifications().update_fcm_device_token(id, None)
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
            cmds.chat().push_notifications().update_fcm_device_token(id, Some(token_clone))
        })?;

        Ok(())
    }

    pub async fn reset_pending_notification(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().push_notifications().reset_pending_notification(id)
        })
    }

    pub async fn get_and_reset_pending_notification_with_device_token(
        &mut self,
        token: FcmDeviceToken,
    ) -> Result<PendingNotification, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().push_notifications().get_and_reset_pending_notification_with_device_token(token)
        })
    }

    pub async fn enable_push_notification_sent_flag(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().push_notifications().enable_push_notification_sent_flag(id)
        })
    }

    pub async fn get_push_notification_state_info_and_add_notification_value(
        &mut self,
        id: AccountIdInternal,
        notification: PendingNotification,
    ) -> Result<PushNotificationStateInfo, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().push_notifications().get_push_notification_state_info_and_add_notification_value(id, notification)
        })
    }
}
