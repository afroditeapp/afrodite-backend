use database::current::write::GetDbWriteCommandsCommon;
use model::{AccountIdInternal, PendingAppNotification, PendingAppNotificationType};

use crate::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsCommonNotification);

impl WriteCommandsCommonNotification<'_> {
    pub async fn upsert_pending_app_notification(
        &self,
        id: AccountIdInternal,
        type_number: PendingAppNotificationType,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .notification()
                .upsert_pending_app_notification(id, type_number)
        })?;

        Ok(())
    }

    pub async fn mark_pending_app_notifications_push_sent(
        &self,
        id: AccountIdInternal,
        type_numbers: Vec<PendingAppNotificationType>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .notification()
                .mark_pending_app_notifications_push_sent(id, type_numbers)
        })?;

        Ok(())
    }

    pub async fn delete_pending_app_notifications(
        &self,
        id: AccountIdInternal,
        notifications: Vec<PendingAppNotification>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .notification()
                .delete_pending_app_notifications(id, notifications)
        })?;

        Ok(())
    }
}
