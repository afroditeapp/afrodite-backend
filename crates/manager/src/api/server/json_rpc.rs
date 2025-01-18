

use std::net::SocketAddr;
use manager_model::JsonRpcRequest;
use manager_model::JsonRpcRequestType;
use manager_model::JsonRpcResponse;
use tracing::info;
use crate::api::client::ClientConfig;
use crate::api::client::ManagerClient;
use crate::api::server::ConnectionUtilsWrite;
use crate::api::GetConfig;
use crate::config::Config;

use tracing::error;

use crate::server::app::S;

use error_stack::{Result, ResultExt};
use super::ClientConnectionReadWrite;
use super::ConnectionUtilsRead;
use super::ServerError;

pub mod software;
pub mod secure_storage;
pub mod system_info;


#[derive(thiserror::Error, Debug)]
pub enum JsonRpcError {
    #[error("Secure storage encryption key not found")]
    SecureStorageEncryptionKeyNotFound,
    #[error("Secure storage encryption key reading failed")]
    SecureStorageEncryptionKeyRead,
    #[error("System info error")]
    SystemInfo,
}


pub async fn handle_json_rpc<
    C: ClientConnectionReadWrite,
>(
    mut c: C,
    address: SocketAddr,
    state: S,
) -> Result<(), ServerError> {
    let request = c.receive_json_rpc_request()
        .await
        .change_context(ServerError::JsonRpcRequestReceivingFailed)?;

    let response = handle_request(
        request,
        address,
        &state,
    ).await?;

    c.send_json_rpc_response(response)
        .await
        .change_context(ServerError::JsonRpcResponseSendingFailed)?;

    Ok(())
}

async fn handle_request(
    request: JsonRpcRequest,
    address: SocketAddr,
    state: &S,
) -> Result<JsonRpcResponse, ServerError> {
    if request.receiver == state.config().manager_name() {
        info!("Running RPC {:?} from {}", &request.request, address);
        handle_request_type(
            request.request,
            state.config()
        )
            .await
            .change_context(ServerError::JsonRpcFailed)
    } else if let Some(m) = state.config().find_remote_manager(&request.receiver)  {
        info!("Forwarding RPC {:?} from {}", &request.request, address);
        let config = ClientConfig {
            url: m.url.clone(),
            root_certificate: state.config().root_certificate(),
            api_key: state.config().api_key().to_string(),
        };
        let client = ManagerClient::connect(config)
            .await
            .change_context(ServerError::Client)?;
        let response = client.send_request(request)
            .await
            .change_context(ServerError::Client)?;
        Ok(response)
    } else {
        Ok(JsonRpcResponse::request_receiver_not_found())
    }
}

pub async fn handle_request_type(
    request: JsonRpcRequestType,
    config: &Config,
) -> Result<JsonRpcResponse, JsonRpcError> {
    match request {
        JsonRpcRequestType::GetManagerInstanceNames =>
            system_info::get_manager_instance_names(config).await,
        JsonRpcRequestType::GetSecureStorageEncryptionKey(name) =>
            secure_storage::get_secure_storage_encryption_key(
                config,
                name,
            ).await,
        JsonRpcRequestType::GetSystemInfo =>
            system_info::get_system_info(config).await,
        JsonRpcRequestType::GetSoftwareUpdateStatus =>
            software::get_software_update_status(config).await,
        JsonRpcRequestType::TriggerSoftwareUpdateDownload =>
            software::trigger_software_update_download(config).await,
        JsonRpcRequestType::TriggerSoftwareUpdateInstall =>
            software::trigger_software_update_install(config).await,
        JsonRpcRequestType::TriggerSystemReboot =>
            software::trigger_system_reboot(config).await,
        JsonRpcRequestType::TriggerBackendDataReset =>
            software::trigger_backend_data_reset(config).await,
        JsonRpcRequestType::ScheduleReboot =>
            software::schedule_reboot(config).await,
    }
}
