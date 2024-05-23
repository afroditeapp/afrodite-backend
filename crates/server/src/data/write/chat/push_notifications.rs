


use database::current::write::chat::ChatStateChanges;
use error_stack::ResultExt;
use model::{AccountId, AccountIdInternal, ChatStateRaw, MessageNumber, PendingMessageId, SyncVersionUtils};
use simple_backend_database::diesel_db::DieselDatabaseError;
use simple_backend_utils::ContextExt;

use crate::{
    data::{cache::CacheError, write::db_transaction, DataError},
    result::Result,
};

define_write_commands!(WriteCommandsChatPushNotifications);

impl WriteCommandsChatPushNotifications<'_> {
    pub async fn remove_device_token(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        self.write_cache(id, |cache| {
            if let Some(chat_state) = cache.chat.as_mut() {
                chat_state.device_token = None;
            } else {
                return Err(CacheError::FeatureNotEnabled.into());
            }

            Ok(())
        }).await?;

        Ok(())
    }
}
