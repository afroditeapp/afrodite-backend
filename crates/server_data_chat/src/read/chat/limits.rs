use database_chat::current::read::GetDbReadCommandsChat;
use model::AccountIdInternal;
use model_chat::DailyLikesLeftInternal;
use server_data::{DataError, define_cmd_wrapper_read, read::DbRead, result::Result};

define_cmd_wrapper_read!(ReadCommandsChatLimits);

impl ReadCommandsChatLimits<'_> {
    pub async fn daily_likes_left_internal(
        &self,
        account: AccountIdInternal,
    ) -> Result<DailyLikesLeftInternal, DataError> {
        let data = self
            .db_read(move |mut cmds| cmds.chat().limits().daily_likes_left(account))
            .await?;
        Ok(data)
    }
}
