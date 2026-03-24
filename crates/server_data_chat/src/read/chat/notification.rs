use database_chat::current::read::GetDbReadCommandsChat;
use model::{AccountIdInternal, NewMessagePushNotificationList, UnixTime};
use model_chat::{
    ChatAppNotificationSettings, ChatEmailNotificationSettings, PendingChatNotification,
};
use server_data::{
    DataError, IntoDataError, cache::CacheReadCommon, define_cmd_wrapper_read, read::DbRead,
    result::Result,
};

define_cmd_wrapper_read!(ReadCommandsChatNotification);

impl ReadCommandsChatNotification<'_> {
    pub async fn chat_app_notification_settings(
        &self,
        id: AccountIdInternal,
    ) -> Result<ChatAppNotificationSettings, DataError> {
        self.read_cache_common(id, |entry| Ok(entry.app_notification_settings.chat))
            .await
            .into_error()
    }

    pub async fn chat_email_notification_settings(
        &self,
        id: AccountIdInternal,
    ) -> Result<ChatEmailNotificationSettings, DataError> {
        self.db_read(move |mut cmds| cmds.chat().notification().email_notification_settings(id))
            .await
            .into_error()
    }

    pub async fn new_message_notification_list(
        &self,
        id: AccountIdInternal,
    ) -> Result<NewMessagePushNotificationList, DataError> {
        self.db_read(move |mut cmds| cmds.chat().notification().new_message_notification_list(id))
            .await
            .into_error()
    }

    pub async fn messages_without_sent_email_notification(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<UnixTime>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .notification()
                .messages_without_sent_email_notification(id)
        })
        .await
        .into_error()
    }

    pub async fn has_sent_message_email_notification(
        &self,
        id: AccountIdInternal,
    ) -> Result<bool, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .notification()
                .has_sent_message_email_notification(id)
        })
        .await
        .into_error()
    }

    pub async fn pending_chat_notifications(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<PendingChatNotification>, DataError> {
        self.db_read(move |mut cmds| cmds.chat().notification().pending_chat_notifications(id))
            .await
            .into_error()
    }
}
