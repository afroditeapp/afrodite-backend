use database_chat::current::write::GetDbWriteCommandsChat;
use model::AccountIdInternal;
use model_chat::ChatPrivacySettings;
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsChatPrivacy);

impl WriteCommandsChatPrivacy<'_> {
    pub async fn upsert_privacy_settings(
        &self,
        id: AccountIdInternal,
        value: ChatPrivacySettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.chat().privacy().upsert_privacy_settings(id, value)
        })
    }
}
