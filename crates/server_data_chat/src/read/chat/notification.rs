use database_chat::current::read::GetDbReadCommandsChat;
use model::{AccountIdInternal, NotificationEvent};
use model_chat::ChatAppNotificationSettings;
use server_data::{define_cmd_wrapper_read, read::DbRead, result::Result, DataError};

define_cmd_wrapper_read!(ReadCommandsChatNotification);

impl ReadCommandsChatNotification<'_> {
    pub async fn chat_app_notification_settings(
        &self,
        id: AccountIdInternal,
    ) -> Result<ChatAppNotificationSettings, DataError> {
        let state = self
            .db_read(move |mut cmds| cmds.chat().notification().app_notification_settings(id))
            .await?;
        Ok(state)
    }

    /// Get [NotificationEvent::NewMessageReceived] if settings allows that
    pub async fn messages(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<NotificationEvent>, DataError> {
        if self.chat_app_notification_settings(id).await?.messages {
            Ok(Some(NotificationEvent::NewMessageReceived))
        } else {
            Ok(None)
        }
    }

    /// Get [NotificationEvent::ReceivedLikesChanged] if settings allows that
    pub async fn likes(
        &self,
        id: AccountIdInternal,
    ) -> Result<Option<NotificationEvent>, DataError> {
        if self.chat_app_notification_settings(id).await?.likes {
            Ok(Some(NotificationEvent::ReceivedLikesChanged))
        } else {
            Ok(None)
        }
    }
}
