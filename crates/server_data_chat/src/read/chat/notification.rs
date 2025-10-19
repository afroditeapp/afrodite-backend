use database_chat::current::read::GetDbReadCommandsChat;
use model::{
    AccountIdInternal, NewMessageNotificationList, PendingMessageIdInternal,
    PendingMessageIdInternalAndMessageTime, ReceivedLikeId, UnixTime,
};
use model_chat::{ChatAppNotificationSettings, ChatEmailNotificationSettings};
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
    ) -> Result<(NewMessageNotificationList, Vec<PendingMessageIdInternal>), DataError> {
        self.db_read(move |mut cmds| cmds.chat().message().new_message_notification_list(id))
            .await
            .into_error()
    }

    pub async fn messages_without_sent_email_notification(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<PendingMessageIdInternalAndMessageTime>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .message()
                .messages_without_sent_email_notification(id)
        })
        .await
        .into_error()
    }

    pub async fn unviewed_received_likes_without_sent_email_notification(
        &self,
        id_receiver: AccountIdInternal,
    ) -> Result<Vec<(ReceivedLikeId, UnixTime)>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.chat()
                .interaction()
                .unviewed_received_likes_without_sent_email_notification(id_receiver)
        })
        .await
        .into_error()
    }
}
