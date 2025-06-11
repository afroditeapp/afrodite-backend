
use database::current::read::GetDbReadCommandsCommon;
use model::{AccountIdInternal, AdminNotification};

use crate::{
    define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError
};

define_cmd_wrapper_read!(ReadCommandsCommonAdminNotification);

impl ReadCommandsCommonAdminNotification<'_> {
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

    pub async fn get_accounts_with_some_wanted_subscriptions(
        &self,
        wanted: AdminNotification,
    ) -> Result<Vec<(AccountIdInternal, AdminNotification)>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common_admin()
                .notification()
                .get_accounts_with_some_wanted_subscriptions(wanted)
        })
        .await
        .into_error()
    }
}
