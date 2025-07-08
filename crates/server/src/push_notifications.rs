use error_stack::ResultExt;
use fcm::message::{AndroidConfig, AndroidMessagePriority, Message, Target};
use model::{
    AccountIdInternal, ClientType, FcmDeviceToken, PendingNotificationFlags,
    PushNotificationStateInfoWithFlags,
};
use serde_json::json;
use server_api::{
    app::{ReadData, WriteData},
    db_write_raw,
};
use server_common::push_notifications::{
    PushNotification, PushNotificationError, PushNotificationStateProvider, SuccessfulSendingAction,
};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use server_data_chat::write::GetWriteCommandsChat;
use server_state::S;

mod ios;

pub struct ServerPushNotificationStateProvider {
    state: S,
}

impl ServerPushNotificationStateProvider {
    pub fn new(state: S) -> Self {
        Self { state }
    }
}

impl PushNotificationStateProvider for ServerPushNotificationStateProvider {
    async fn get_push_notification_state_info_and_add_notification_value(
        &self,
        account_id: AccountIdInternal,
    ) -> error_stack::Result<PushNotificationStateInfoWithFlags, PushNotificationError> {
        let flags = self
            .state
            .read()
            .common()
            .cached_pending_notification_flags(account_id)
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::ReadingNotificationFlagsFromCacheFailed)?;

        if flags.is_empty() {
            return Ok(PushNotificationStateInfoWithFlags::EmptyFlags);
        }

        let info = db_write_raw!(self.state, move |cmds| {
            cmds.common()
                .push_notification()
                .get_push_notification_state_info_and_add_notification_value(
                    account_id,
                    flags.into(),
                )
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(PushNotificationError::SettingPushNotificationSentFlagFailed)?;

        Ok(PushNotificationStateInfoWithFlags::WithFlags { info, flags })
    }

    async fn enable_push_notification_sent_flag(
        &self,
        account_id: AccountIdInternal,
    ) -> error_stack::Result<(), PushNotificationError> {
        db_write_raw!(self.state, move |cmds| {
            cmds.common()
                .push_notification()
                .enable_push_notification_sent_flag(account_id)
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(PushNotificationError::SettingPushNotificationSentFlagFailed)?;

        Ok(())
    }

    async fn remove_device_token(
        &self,
        account_id: AccountIdInternal,
    ) -> error_stack::Result<(), PushNotificationError> {
        db_write_raw!(self.state, move |cmds| {
            cmds.common()
                .push_notification()
                .remove_fcm_device_token(account_id)
                .await
        })
        .await
        .map_err(|e| e.into_report())
        .change_context(PushNotificationError::RemoveDeviceTokenFailed)
    }

    async fn remove_specific_notification_flags_from_cache(
        &self,
        account_id: AccountIdInternal,
        flags: PendingNotificationFlags,
    ) -> error_stack::Result<(), PushNotificationError> {
        self.state
            .read()
            .cache_read_write_access()
            .write_cache(account_id, move |entry| {
                entry.common.pending_notification_flags -= flags;
                Ok(())
            })
            .await
            .map_err(|e| e.into_error())
            .change_context(PushNotificationError::RemoveSpecificNotificationFlagsFromCacheFailed)
    }

    async fn save_current_non_empty_notification_flags_from_cache_to_database(
        &self,
    ) -> error_stack::Result<(), PushNotificationError> {
        let account_ids = self
            .state
            .read()
            .common()
            .account_ids_internal_vec()
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::SaveToDatabaseFailed)?;

        for account_id in account_ids {
            db_write_raw!(self.state, move |cmds| {
                cmds.common()
                    .push_notification()
                    .save_current_non_empty_notification_flags_from_cache_to_database(account_id)
                    .await
            })
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::SaveToDatabaseFailed)?;
        }

        Ok(())
    }

    async fn convert_to_push_notifications(
        &self,
        account_id: AccountIdInternal,
        token: FcmDeviceToken,
        flags: PendingNotificationFlags,
    ) -> error_stack::Result<Vec<PushNotification>, PushNotificationError> {
        let platform = self
            .state
            .read()
            .common()
            .client_config()
            .client_login_session_platform(account_id)
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::CreatingPushNotificationsFailed)?;

        if let Some(ClientType::Ios) = platform {
            // On iOS, data notifications don't work when app is closed
            // from task switcher and are not reliable, so send visible
            // notifications.
            ios::ios_notifications(&self.state, account_id, token, flags)
                .await
                .change_context(PushNotificationError::CreatingPushNotificationsFailed)
        } else {
            let message = Message {
                // Use minimal notification data as this only triggers client
                // to download the notification.
                data: Some(json!({
                    "n": "",
                })),
                target: Target::Token(token.into_string()),
                android: Some(AndroidConfig {
                    priority: Some(AndroidMessagePriority::High),
                    ..Default::default()
                }),
                apns: None,
                webpush: None,
                fcm_options: None,
                notification: None,
            };

            Ok(vec![PushNotification {
                message: Some(message),
                flags,
                successful_sending_action: None,
            }])
        }
    }

    async fn handle_successful_message_sending_action(
        &self,
        action: SuccessfulSendingAction,
    ) -> error_stack::Result<(), PushNotificationError> {
        match action {
            SuccessfulSendingAction::MarkMessageNotificationSent { message } => {
                db_write_raw!(self.state, move |cmds| {
                    cmds.chat()
                        .mark_receiver_push_notification_sent(vec![message])
                        .await
                })
                .await
                .map_err(|e| e.into_report())
                .change_context(
                    PushNotificationError::HandlingSuccessfulMessageSendingActionFailed,
                )?;
            }
        }

        Ok(())
    }
}
