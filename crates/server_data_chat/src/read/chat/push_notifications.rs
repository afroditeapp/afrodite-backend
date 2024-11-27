use model::AccountIdInternal;
use server_data::{
    define_cmd_wrapper, result::Result, DataError
};

use crate::read::DbReadChat;

define_cmd_wrapper!(ReadCommandsChatPushNotifications);

impl<C: DbReadChat> ReadCommandsChatPushNotifications<C> {
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
