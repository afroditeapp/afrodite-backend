use database_chat::current::read::GetDbReadCommandsChat;
use model::AccountIdInternal;
use server_data::{define_cmd_wrapper_read, read::DbRead, result::Result, DataError};

define_cmd_wrapper_read!(ReadCommandsChatPushNotifications);

impl ReadCommandsChatPushNotifications<'_> {
    pub async fn push_notification_already_sent(
        &self,
        id: AccountIdInternal,
    ) -> Result<bool, DataError> {
        let chat_state = self
            .db_read(move |mut cmds| cmds.chat().chat_state(id))
            .await?;
        Ok(chat_state.fcm_notification_sent)
    }
}
