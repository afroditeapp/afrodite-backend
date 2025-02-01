

use error_stack::ResultExt;
use manager_model::JsonRpcResponse;
use manager_model::ManagerInstanceName;
use crate::api::GetConfig;
use crate::server::info::SystemInfoGetter;

use error_stack::Result;

use super::JsonRpcError;

pub trait RpcSystemInfo: GetConfig {
    async fn rpc_get_manager_instance_names(&self) -> Result<JsonRpcResponse, JsonRpcError> {
        let mut accessible_instances = vec![ManagerInstanceName::new(self.config().manager_name().to_string())];

        let remote_managers = self.config()
            .remote_managers()
            .iter()
            .map(|v| v.manager_name.clone());

        accessible_instances.extend(remote_managers);

        Ok(JsonRpcResponse::manager_instance_names(accessible_instances))
    }

    async fn rpc_get_system_info(&self) -> Result<JsonRpcResponse, JsonRpcError> {
        let info = SystemInfoGetter::system_info(self.config())
            .await
            .change_context(JsonRpcError::SystemInfo)?;
        Ok(JsonRpcResponse::system_info(info))
    }
}

impl <T: GetConfig> RpcSystemInfo for T {}
