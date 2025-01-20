

use manager_model::JsonRpcResponse;
use crate::api::GetRebootManager;

use error_stack::{Result, ResultExt};

use super::JsonRpcError;

pub trait RpcReboot: GetRebootManager {
    async fn rpc_trigger_system_reboot(
        &self,
    ) -> Result<JsonRpcResponse, JsonRpcError> {
        self.reboot_manager()
            .reboot_now()
            .await
            .change_context(JsonRpcError::RebootManager)?;
        Ok(JsonRpcResponse::successful())
    }
}

impl <T: GetRebootManager> RpcReboot for T {}
