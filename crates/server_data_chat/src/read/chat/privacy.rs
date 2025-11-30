use database_chat::current::read::GetDbReadCommandsChat;
use model::AccountIdInternal;
use model_chat::ChatPrivacySettings;
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsChatPrivacy);

impl ReadCommandsChatPrivacy<'_> {
    pub async fn chat_privacy_settings(
        &self,
        id: AccountIdInternal,
    ) -> Result<ChatPrivacySettings, DataError> {
        self.db_read(move |mut cmds| cmds.chat().privacy().privacy_settings(id))
            .await
            .into_error()
    }
}
