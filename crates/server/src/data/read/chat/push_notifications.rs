use model::{
    AccountIdInternal, AccountInteractionState, ChatStateRaw, FcmDeviceToken, MatchesPage, MessageNumber, PendingMessagesPage, ReceivedBlocksPage, ReceivedLikesPage, SentBlocksPage, SentLikesPage
};

use crate::{
    data::{cache::CacheError, DataError, IntoDataError},
    result::Result,
};

use super::ReadCommands;

define_read_commands!(ReadCommandsChatPushNotifications);


impl ReadCommandsChatPushNotifications<'_> {
    pub async fn push_notification_already_sent(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<bool, DataError> {
        let chat_state = self.db_read(move |mut cmds| cmds.chat().chat_state(id)).await?;
        Ok(chat_state.fcm_notification_sent)
    }
}
