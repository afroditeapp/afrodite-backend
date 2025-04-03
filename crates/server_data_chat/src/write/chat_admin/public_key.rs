use database_chat::current::write::GetDbWriteCommandsChat;
use model::AccountIdInternal;
use server_data::{define_cmd_wrapper_write, DataError, result::Result};
use server_data::write::DbTransaction;

define_cmd_wrapper_write!(WriteCommandsChatAdminPublicKey);

impl WriteCommandsChatAdminPublicKey<'_> {
    pub async fn set_max_public_key_count(
        &self,
        id: AccountIdInternal,
        count: i64,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat_admin().public_key().set_max_public_key_count(id, count)
        })
    }
}
