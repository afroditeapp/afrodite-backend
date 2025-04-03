use database_chat::current::read::GetDbReadCommandsChat;
use model::{AccountIdInternal, PublicKeyVersion};
use model_chat::{GetPrivatePublicKeyInfo, GetPublicKey};
use server_data::{db_manager::InternalReading, define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError};

define_cmd_wrapper_read!(ReadCommandsChatPublicKey);

impl ReadCommandsChatPublicKey<'_> {
    pub async fn get_public_key(
        &self,
        id: AccountIdInternal,
        version: PublicKeyVersion,
    ) -> Result<GetPublicKey, DataError> {
        self.db_read(move |mut cmds| cmds.chat().public_key().public_key(id, version))
            .await
            .map(|key| GetPublicKey { key })
            .into_error()
    }

    pub async fn get_private_public_key_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<GetPrivatePublicKeyInfo, DataError> {
        let account_specific_value = self
            .db_read(move |mut cmds| cmds.chat().public_key().max_public_key_count_account_config(id))
            .await?;

        let config_value = self.config().limits_chat().max_public_key_count;

        Ok(GetPrivatePublicKeyInfo {
            max_public_key_count_from_backend_config: config_value.into(),
            max_public_key_count_from_account_config: account_specific_value,
        })
    }
}
