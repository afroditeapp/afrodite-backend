use database_chat::current::write::GetDbWriteCommandsChat;
use model::{AccountIdInternal, PendingMessageIdInternal};
use model_chat::{ChatAppNotificationSettings, ChatEmailNotificationSettings};
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

    pub async fn mark_receiver_push_notification_sent(
        &self,
        messages: Vec<PendingMessageIdInternal>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .message()
                .mark_receiver_push_notification_sent(messages)
        })
    }

    pub async fn mark_message_email_notification_sent(
        &self,
        messages: Vec<PendingMessageIdInternal>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat()
                .message()
                .mark_message_email_notification_sent(messages)
        })
    }
}
