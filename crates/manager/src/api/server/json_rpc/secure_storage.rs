
use error_stack::report;
use manager_model::JsonRpcResponse;
use manager_model::ManagerInstanceName;

use crate::api::GetConfig;

use error_stack::{Result, ResultExt};

use super::JsonRpcError;

pub trait RpcSecureStorage: GetConfig {
    async fn rpc_get_secure_storage_encryption_key(
        &self,
        name: ManagerInstanceName,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        let key = self.config()
            .encryption_keys()
            .iter()
            .find(|s| s.manager_name == name)
            .ok_or_else(|| report!(JsonRpcError::SecureStorageEncryptionKeyNotFound))?;

        let key = key.read_encryption_key()
            .await
            .change_context(JsonRpcError::SecureStorageEncryptionKeyRead)?;

        Ok(JsonRpcResponse::secure_storage_encryption_key(key))
    }
}

impl <T: GetConfig> RpcSecureStorage for T {}
