use database::current::read::GetDbReadCommandsCommon;
use model::{
    AccountIdInternal, PendingNotificationFlags, PushNotificationDbState,
    PushNotificationStateInfo, PushNotificationStateInfoWithFlags,
};
use server_common::data::IntoDataError;

use crate::{
    DataError, cache::CacheReadCommon, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsCommonPushNotification);

impl ReadCommandsCommonPushNotification<'_> {
    pub async fn cached_pending_notification_flags(
        &self,
        id: AccountIdInternal,
    ) -> Result<PendingNotificationFlags, DataError> {
        let flags = self
            .read_cache_common(id, |cache| Ok(cache.pending_notification_flags))
            .await?;
        Ok(flags)
    }

    pub async fn fcm_token_exists(&self, id: AccountIdInternal) -> Result<bool, DataError> {
        self.push_notification_db_state(id)
            .await
            .map(|v| v.fcm_device_token.is_some())
    }

    pub async fn push_notification_db_state(
        &self,
        id: AccountIdInternal,
    ) -> Result<PushNotificationDbState, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common()
                .push_notification()
                .push_notification_db_state(id)
        })
        .await
        .into_error()
    }

    pub async fn push_notification_state(
        &self,
        id: AccountIdInternal,
    ) -> Result<PushNotificationStateInfoWithFlags, DataError> {
        let db_state = self
            .db_read(move |mut cmds| {
                cmds.common()
                    .push_notification()
                    .push_notification_db_state(id)
            })
            .await
            .into_error()?;

        // Cache contains the latest state
        let flags = self.cached_pending_notification_flags(id).await?;

        if flags.is_empty() {
            Ok(PushNotificationStateInfoWithFlags::EmptyFlags)
        } else {
            Ok(PushNotificationStateInfoWithFlags::WithFlags {
                info: PushNotificationStateInfo {
                    fcm_device_token: db_state.fcm_device_token,
                },
                flags,
            })
        }
    }
}
