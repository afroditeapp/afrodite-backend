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
    pub async fn device_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<FcmDeviceToken>, DataError> {
        let token = self.read_cache(id, |cache| {
            let chat_state = cache.chat
                .as_ref()
                .ok_or(CacheError::FeatureNotEnabled)?;
            error_stack::Result::<_, CacheError>::Ok(chat_state.device_token.clone())
        }).await??;

        Ok(token)
    }
}
