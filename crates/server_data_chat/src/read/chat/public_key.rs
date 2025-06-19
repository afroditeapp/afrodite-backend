use database_chat::current::read::GetDbReadCommandsChat;
use model::{AccountIdInternal, PublicKeyId};
use model_chat::{GetLatestPublicKeyId, GetPrivatePublicKeyInfo};
use server_data::{
    DataError, IntoDataError, db_manager::InternalReading, define_cmd_wrapper_read, read::DbRead,
    result::Result,
};

define_cmd_wrapper_read!(ReadCommandsChatPublicKey);

impl ReadCommandsChatPublicKey<'_> {
    pub async fn get_public_key_data(
        &self,
        id: AccountIdInternal,
        key_id: PublicKeyId,
    ) -> Result<Option<Vec<u8>>, DataError> {
        self.db_read(move |mut cmds| cmds.chat().public_key().public_key_data(id, key_id))
            .await
            .into_error()
    }

    pub async fn get_latest_public_key_id(
        &self,
        id: AccountIdInternal,
    ) -> Result<GetLatestPublicKeyId, DataError> {
        let latest_public_key_id = self
            .db_read(move |mut cmds| cmds.chat().public_key().latest_public_key_id(id))
            .await?;

        Ok(GetLatestPublicKeyId {
            id: latest_public_key_id,
        })
    }

    pub async fn get_private_public_key_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<GetPrivatePublicKeyInfo, DataError> {
        let (latest_public_key_id, account_specific_value) = self
            .db_read(move |mut cmds| {
                let latest_public_key_id = cmds.chat().public_key().latest_public_key_id(id)?;
                let limit = cmds
                    .chat()
                    .public_key()
                    .max_public_key_count_account_config(id)?;
                Ok((latest_public_key_id, limit))
            })
            .await?;

        let config_value = self.config().limits_chat().max_public_key_count;

        Ok(GetPrivatePublicKeyInfo {
            latest_public_key_id,
            max_public_key_count_from_backend_config: config_value.into(),
            max_public_key_count_from_account_config: account_specific_value,
        })
    }
}
