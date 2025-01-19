

use error_stack::ResultExt;
use manager_model::JsonRpcResponse;
use manager_model::ManagerInstanceName;
use crate::config::Config;
use crate::server::info::SystemInfoGetter;

use error_stack::Result;

use super::JsonRpcError;

pub async fn get_manager_instance_names(
    config: &Config,
) -> Result<JsonRpcResponse, JsonRpcError> {
    let current_manager = [ManagerInstanceName::new(config.manager_name().to_string())];

    let accessible_instances = config
        .remote_managers()
        .iter()
        .map(|v| v.manager_name.clone())
        .chain(current_manager)
        .collect::<Vec<ManagerInstanceName>>();

    Ok(JsonRpcResponse::manager_instance_names(accessible_instances))
}

pub async fn get_system_info(
    config: &Config,
) -> Result<JsonRpcResponse, JsonRpcError> {
    let info = SystemInfoGetter::system_info(config)
        .await
        .change_context(JsonRpcError::SystemInfo)?;
    Ok(JsonRpcResponse::system_info(info))
}
