use database_chat::current::write::GetDbWriteCommandsChat;
use model::AccountIdInternal;
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsChatLimits);

impl WriteCommandsChatLimits<'_> {
    pub async fn reset_daily_likes_left(
        &self,
        id: AccountIdInternal,
        value: i16,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().limits().reset_daily_likes_left(id, value)
        })?;
        Ok(())
    }

    pub async fn decrement_daily_likes_left(&self, id: AccountIdInternal) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().limits().decrement_daily_likes_left(id)
        })?;
        Ok(())
    }

    pub async fn reset_daily_likes_left_sync_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().limits().reset_daily_likes_left_sync_version(id)
        })?;
        Ok(())
    }
}
