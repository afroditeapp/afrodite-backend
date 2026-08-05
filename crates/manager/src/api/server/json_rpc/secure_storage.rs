use error_stack::{IntoReport, ResultExt};
use manager_model::{JsonRpcResponse, ManagerInstanceName};
use simple_backend_utils::Result;

use super::JsonRpcError;
use crate::api::GetConfig;

pub trait RpcSecureStorage: GetConfig {
    async fn rpc_get_secure_storage_encryption_key(
        &self,
        name: ManagerInstanceName,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        let key = self
            .config()
            .encryption_keys()
            .iter()
            .find(|s| s.manager_name == name)
            .ok_or_else(|| JsonRpcError::SecureStorageEncryptionKeyNotFound.into_report())?;

        let key = key
            .read_encryption_key()
            .await
            .change_context(JsonRpcError::SecureStorageEncryptionKeyRead)?;

        Ok(JsonRpcResponse::secure_storage_encryption_key(key))
    }
}

impl<T: GetConfig> RpcSecureStorage for T {}
