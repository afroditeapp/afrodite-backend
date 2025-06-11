use database::current::write::GetDbWriteCommandsCommon;
use model::{AccountIdInternal, AdminNotification};

use crate::{
    define_cmd_wrapper_write, result::Result, write::db_transaction, DataError
};

use crate::write::DbTransaction;

define_cmd_wrapper_write!(WriteCommandsCommonAdminNotification);

impl WriteCommandsCommonAdminNotification<'_> {
    pub async fn set_admin_notification_subscriptions(
        &self,
        id: AccountIdInternal,
        subscriptions: AdminNotification,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common_admin().notification().set_admin_notification_subscriptions(id, subscriptions)
        })
    }
}
