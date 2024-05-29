use model::AccountIdInternal;

use server_data::{define_server_data_read_commands, result::Result, DataError};

define_server_data_read_commands!(ReadCommandsChatPushNotifications);
define_db_read_command!(ReadCommandsChatPushNotifications);

impl ReadCommandsChatPushNotifications<'_> {
    pub async fn push_notification_already_sent(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<bool, DataError> {
        let chat_state = self
            .db_read(move |mut cmds| cmds.chat().chat_state(id))
            .await?;
        Ok(chat_state.fcm_notification_sent)
    }
}
