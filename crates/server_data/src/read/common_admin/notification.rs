
use database::current::read::GetDbReadCommandsCommon;
use model::{AccountIdInternal, AdminNotificationSubscriptions};

use crate::{
    define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError
};

define_cmd_wrapper_read!(ReadCommandsCommonAdminNotification);

impl ReadCommandsCommonAdminNotification<'_> {
    pub async fn admin_notification_subscriptions(
        &self,
        id: AccountIdInternal,
    ) -> Result<AdminNotificationSubscriptions, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common_admin()
                .notification()
                .admin_notification_subscriptions(id)
        })
        .await
        .into_error()
    }
}
