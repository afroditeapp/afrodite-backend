

use manager_model::JsonRpcResponse;
use crate::config::Config;

use error_stack::Result;

use super::JsonRpcError;


pub async fn get_software_update_status(
    config: &Config,
) -> Result<JsonRpcResponse, JsonRpcError> {
    // Ok(JsonRpcResponse::software_update_status(status))
    todo!()
}

pub async fn trigger_software_update_download(
    config: &Config,
) -> Result<JsonRpcResponse, JsonRpcError> {
    // Ok(JsonRpcResponse::software_update_status(status))
    todo!()
}

pub async fn trigger_software_update_install(
    config: &Config,
) -> Result<JsonRpcResponse, JsonRpcError> {
    // Ok(JsonRpcResponse::software_update_status(status))
    todo!()
}

pub async fn trigger_system_reboot(
    config: &Config,
) -> Result<JsonRpcResponse, JsonRpcError> {
    // Ok(JsonRpcResponse::software_update_status(status))
    todo!()
}


pub async fn trigger_backend_data_reset(
    config: &Config,
) -> Result<JsonRpcResponse, JsonRpcError> {
    // Ok(JsonRpcResponse::software_update_status(status))
    todo!()
}

pub async fn schedule_reboot(
    config: &Config,
) -> Result<JsonRpcResponse, JsonRpcError> {
    // Ok(JsonRpcResponse::software_update_status(status))
    todo!()
}
