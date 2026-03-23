use database::current::read::GetDbReadCommandsCommon;
use model::{
    AccountIdInternal, PendingAppNotification, PendingAppNotificationDb, PendingAppNotificationType,
};
use server_common::data::IntoDataError;

use crate::{DataError, define_cmd_wrapper_read, read::DbRead, result::Result};

define_cmd_wrapper_read!(ReadCommandsCommonNotification);

impl ReadCommandsCommonNotification<'_> {
    pub async fn pending_app_notification_type_numbers(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<PendingAppNotificationType>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common()
                .notification()
                .pending_app_notification_type_numbers(id)
        })
        .await
        .into_error()
    }

    pub async fn pending_app_notification_type_numbers_without_sent_push(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<PendingAppNotificationType>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common()
                .notification()
                .pending_app_notification_type_numbers_without_sent_push(id)
        })
        .await
        .into_error()
    }

    pub async fn pending_app_notifications_without_sent_push(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<PendingAppNotification>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common()
                .notification()
                .pending_app_notifications_without_sent_push(id)
        })
        .await
        .into_error()
    }

    pub async fn pending_app_notifications(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<PendingAppNotification>, DataError> {
        self.db_read(move |mut cmds| cmds.common().notification().pending_app_notifications(id))
            .await
            .into_error()
    }

    pub async fn received_likes_notification_with_unsent_email(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<PendingAppNotificationDb>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common()
                .notification()
                .received_likes_notification_with_unsent_email(id)
        })
        .await
        .into_error()
    }
}
