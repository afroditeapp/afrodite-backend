use database::current::read::GetDbReadCommandsCommon;
use model::{AccountIdInternal, AdminNotification, AdminNotificationSettings, DayTimestamp};

use crate::{DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result};

define_cmd_wrapper_read!(ReadCommandsCommonAdminNotification);

impl ReadCommandsCommonAdminNotification<'_> {
    pub async fn admin_notification_settings(
        &self,
        id: AccountIdInternal,
    ) -> Result<AdminNotificationSettings, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common_admin()
                .notification()
                .admin_notification_settings(id)
        })
        .await
        .into_error()
    }

    pub async fn nearest_start_time(&self) -> Result<DayTimestamp, DataError> {
        self.db_read(move |mut cmds| cmds.common_admin().notification().nearest_start_time())
            .await
            .into_error()
    }

    pub async fn admin_notification_subscriptions(
        &self,
        id: AccountIdInternal,
    ) -> Result<AdminNotification, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common_admin()
                .notification()
                .admin_notification_subscriptions(id)
        })
        .await
        .into_error()
    }

    pub async fn get_accounts_which_should_receive_notification(
        &self,
        wanted: AdminNotification,
    ) -> Result<Vec<(AccountIdInternal, AdminNotification)>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common_admin()
                .notification()
                .get_accounts_which_should_receive_notification(wanted)
        })
        .await
        .into_error()
    }
}
