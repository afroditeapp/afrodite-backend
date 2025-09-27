use error_stack::ResultExt;
use model::{
    AccountIdInternal, ClientLanguage, PendingNotificationFlags, PushNotificationStateInfoWithFlags,
};
use server_api::{
    app::{ReadData, WriteData},
    db_write_raw,
};
use server_common::push_notifications::{PushNotificationError, PushNotificationStateProvider};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use server_state::S;

mod visibility;

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
            .push_notification()
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
        .change_context(PushNotificationError::ReadingNotificationSentStatusFailed)?;

        Ok(PushNotificationStateInfoWithFlags::WithFlags { info, flags })
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

    async fn is_pending_notification_visible_notification(
        &self,
        account_id: AccountIdInternal,
        flags: PendingNotificationFlags,
    ) -> error_stack::Result<bool, PushNotificationError> {
        visibility::is_notification_visible(&self.state, account_id, flags)
            .await
            .change_context(PushNotificationError::NotificationVisiblityCheckFailed)
    }

    async fn client_language(
        &self,
        account_id: AccountIdInternal,
    ) -> error_stack::Result<ClientLanguage, PushNotificationError> {
        let client_language = self
            .state
            .read()
            .common()
            .client_config()
            .client_language(account_id)
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::GetClientLanguageFailed)?;
        Ok(client_language)
    }
}
