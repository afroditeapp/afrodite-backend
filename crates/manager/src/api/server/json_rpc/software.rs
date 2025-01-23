

use manager_model::{JsonRpcResponse, SoftwareUpdateTaskType};
use crate::api::{GetConfig, GetUpdateManager};

use error_stack::{Result, ResultExt};

use super::JsonRpcError;

pub trait RpcSoftware: GetConfig + GetUpdateManager {
    async fn rpc_get_software_update_status(&self) -> Result<JsonRpcResponse, JsonRpcError> {
        Ok(JsonRpcResponse::software_update_status(self.update_manager().read_state().await))
    }

    async fn rpc_trigger_update_manager_related_action(
        &self,
        message: SoftwareUpdateTaskType,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        self.update_manager().send_message(message)
            .await
            .change_context(JsonRpcError::UpdateManager)?;
        Ok(JsonRpcResponse::successful())
    }
}

impl <T: GetConfig + GetUpdateManager> RpcSoftware for T {}
