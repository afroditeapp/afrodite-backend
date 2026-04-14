use database_chat::current::write::GetDbWriteCommandsChat;
use model::{AccountIdInternal, NewMessagePushNotification};
use model_chat::{
    ChatAppNotificationSettings, ChatEmailNotificationSettings, PendingChatNotificationToDelete,
};
use server_data::{
    DataError, IntoDataError, cache::CacheWriteCommon, db_transaction, define_cmd_wrapper_write,
    result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsChatNotification);

impl WriteCommandsChatNotification<'_> {
    pub async fn upsert_app_notification_settings(
        &self,
        id: AccountIdInternal,
        value: ChatAppNotificationSettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .notification()
                .upsert_app_notification_settings(id, value)
        })?;

        self.write_cache_common(id, |entry| {
            entry.app_notification_settings.chat = value;
            Ok(())
        })
        .await
        .into_error()
    }

    pub async fn upsert_email_notification_settings(
        &self,
        id: AccountIdInternal,
        value: ChatEmailNotificationSettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .notification()
                .upsert_email_notification_settings(id, value)
        })
    }

    pub async fn mark_message_email_notification_sent(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .notification()
                .mark_message_email_notification_sent(id)
        })
    }

    pub async fn upsert_pending_chat_notification(
        &self,
        viewer_id: AccountIdInternal,
        sender_id: AccountIdInternal,
        message_count: i64,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().notification().upsert_pending_chat_notification(
                viewer_id,
                sender_id,
                message_count,
            )
        })
    }

    pub async fn mark_pending_chat_notifications_push_sent(
        &self,
        id: AccountIdInternal,
        notifications: Vec<NewMessagePushNotification>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .notification()
                .mark_pending_chat_notifications_push_sent(id, notifications)
        })
    }

    pub async fn delete_pending_chat_notifications(
        &self,
        id: AccountIdInternal,
        notifications: Vec<PendingChatNotificationToDelete>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .notification()
                .delete_pending_chat_notifications(id, notifications)
        })
    }
}
