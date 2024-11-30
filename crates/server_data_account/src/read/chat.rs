use database_account::current::read::GetDbReadCommandsAccount;
use model::PublicKeyIdAndVersion;
use model_account::AccountIdInternal;
use server_data::{
    define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError,
};

define_cmd_wrapper_read!(ReadCommandsChatUtils);

impl ReadCommandsChatUtils<'_> {
    pub async fn get_latest_public_keys_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<PublicKeyIdAndVersion>, DataError> {
        self.db_read(move |mut cmds| cmds.account_chat_utils().get_latest_public_keys_info(id))
            .await
            .into_error()
    }
}
