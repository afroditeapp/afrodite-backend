use error_stack::ResultExt;
use model::{AccountIdInternal, ClientType, PushNotificationSendingInfo};
use server_api::{
    app::{EventManagerProvider, ReadData, WriteData},
    db_write_raw,
};
use server_common::push_notifications::{PushNotificationError, PushNotificationStateProvider};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use server_state::S;

mod notifications;

#[derive(Clone)]
pub struct ServerPushNotificationStateProvider {
    state: S,
}

impl ServerPushNotificationStateProvider {
    pub fn new(state: S) -> Self {
        Self { state }
    }
}

impl PushNotificationStateProvider for ServerPushNotificationStateProvider {
    async fn get_and_reset_push_notifications(
        &self,
        account_id: AccountIdInternal,
    ) -> error_stack::Result<PushNotificationSendingInfo, PushNotificationError> {
        let db_state = self
            .state
            .read()
            .common()
            .push_notification()
            .push_notification_db_state(account_id)
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::GetAndResetPushNotificationsFailed)?;

        let flags = self
            .state
            .event_manager()
            .remove_pending_notification_flags_from_cache(account_id)
            .await;

        let notifications =
            notifications::notifications_for_sending(&self.state, account_id, flags)
                .await
                .map_err(|e| e.into_report())
                .change_context(PushNotificationError::GetAndResetPushNotificationsFailed)?;

        Ok(PushNotificationSendingInfo {
            db_state,
            notifications,
        })
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

    async fn save_current_notification_flags_to_database_if_needed(
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
                    .save_current_notification_flags_to_database_if_needed(account_id)
                    .await
            })
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::SaveToDatabaseFailed)?;
        }

        Ok(())
    }

    async fn client_type(
        &self,
        account_id: AccountIdInternal,
    ) -> error_stack::Result<Option<ClientType>, PushNotificationError> {
        self.state
            .read()
            .common()
            .client_config()
            .client_login_session_platform(account_id)
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::GetClientType)
    }
}
