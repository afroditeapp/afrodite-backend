use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Path, Query},
    Json,
};
use manager_model::{
    BuildInfo, DataEncryptionKey, DownloadTypeQueryParam, RebootQueryParam, ResetDataQueryParam, ServerNameText, SoftwareInfo, SoftwareOptionsQueryParam, SystemInfo, SystemInfoList
};
use tracing::{info, error};

use super::{utils::StatusCode, GetApiManager, GetConfig, GetUpdateManager};
use crate::server::{info::SystemInfoGetter, update::UpdateDirCreator};

pub const PATH_GET_ENCRYPTION_KEY: &str = "/manager_api/encryption_key/:server";

/// Get encryption key for some server
#[utoipa::path(
    get,
    path = "/manager_api/encryption_key/{server}",
    params(ServerNameText),
    responses(
        (status = 200, description = "Encryption key found.", body = DataEncryptionKey),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn get_encryption_key<S: GetConfig>(
    Path(server): Path<ServerNameText>,
    ConnectInfo(client): ConnectInfo<SocketAddr>,
    state: S,
) -> Result<Json<DataEncryptionKey>, StatusCode> {
    if let Some(s) = state
        .config()
        .encryption_keys()
        .iter()
        .find(|s| s.name == server.server)
    {
        let key = s.read_encryption_key().await?;
        info!("Sending encryption key {} to {}", server.server, client);
        Ok(key.into())
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub const PATH_GET_LATEST_SOFTWARE: &str = "/manager_api/latest_software";

/// Download latest software.
///
/// Returns BuildInfo JSON.
#[utoipa::path(
    get,
    path = "/manager_api/latest_software",
    params(SoftwareOptionsQueryParam, DownloadTypeQueryParam),
    responses(
        (status = 200, description = "UTF-8 JSON", body = Vec<u8>),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn get_latest_software<S: GetConfig + GetApiManager>(
    Query(software): Query<SoftwareOptionsQueryParam>,
    Query(download): Query<DownloadTypeQueryParam>,
    ConnectInfo(client): ConnectInfo<SocketAddr>,
    state: S,
) -> Result<Vec<u8>, StatusCode> {
    if state.config().software_update_provider().is_some() {
        info!(
            "Get latest software request received. Sending {:?} {:?} to {}",
            software.software_options, download.download_type, client,
        );

        let data = BuildInfo {
            // TODO(prod): Download info what is the latest backend release on
            //             GitHub.
            ..Default::default()
        };

        let data = match serde_json::to_vec(&data) {
            Ok(data) => data,
            Err(e) => {
                error!("Error: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        Ok(data)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub const PATH_POST_RQUEST_SOFTWARE_UPDATE: &str = "/manager_api/request_software_update";

/// Request software update.
///
/// Manager will update the requested software and reboot the computer as soon
/// as possible if specified.
///
/// Software's current data storage can be resetted. This will move
/// the data in the data storage to another location waiting for deletion.
/// The deletetion will happen when the next data reset happens.
/// The selected software must support data reset_data query parameter.
/// Resetting the data storage can only work if
/// it is configured from manager config file.
#[utoipa::path(
    post,
    path = "/manager_api/request_software_update",
    params(SoftwareOptionsQueryParam, RebootQueryParam, ResetDataQueryParam),
    responses(
        (status = 200, description = "Request received"),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn post_request_software_update<S: GetConfig + GetUpdateManager>(
    Query(software): Query<SoftwareOptionsQueryParam>,
    Query(reboot): Query<RebootQueryParam>,
    Query(reset_data): Query<ResetDataQueryParam>,
    ConnectInfo(client): ConnectInfo<SocketAddr>,
    state: S,
) -> Result<(), StatusCode> {
    info!(
        "Update software request received from {}. Software {:?}, reboot {:?}, reset_data {:?}",
        client, software.software_options, reboot.reboot, reset_data.reset_data,
    );

    state
        .update_manager()
        .send_update_request(software.software_options, reboot.reboot, reset_data)
        .await?;

    Ok(())
}

pub const PATH_POST_RQUEST_RESTART_OR_RESET_BACKEND: &str =
    "/manager_api/request_restart_or_reset_backend";

/// Restart or reset backend.
///
/// Restarts backend process. Optionally backend data storage can be reset
/// also. The data reset will work as described in request_software_update
/// request documentation.
#[utoipa::path(
    post,
    path = "/manager_api/request_restart_or_reset_backend",
    params(ResetDataQueryParam),
    responses(
        (status = 200, description = "Restart or reset request received"),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn post_request_restart_or_reset_backend<S: GetConfig + GetUpdateManager>(
    Query(reset_data): Query<ResetDataQueryParam>,
    ConnectInfo(client): ConnectInfo<SocketAddr>,
    state: S,
) -> Result<(), StatusCode> {
    info!(
        "Backend restart request received from {}. reset_data {:?}",
        client, reset_data.reset_data,
    );

    state
        .update_manager()
        .send_restart_backend_request(reset_data)
        .await?;

    Ok(())
}

pub const PATH_GET_SOFTWARE_INFO: &str = "/manager_api/software_info";

/// Get current software info about currently installed backend and manager.
#[utoipa::path(
    get,
    path = "/manager_api/software_info",
    responses(
        (status = 200, description = "Software info", body = SoftwareInfo),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn get_software_info<S: GetConfig>(
    ConnectInfo(client): ConnectInfo<SocketAddr>,
    state: S,
) -> Result<Json<SoftwareInfo>, StatusCode> {
    info!("Get current software info received from {}.", client,);

    let info = UpdateDirCreator::current_software(state.config()).await?;
    Ok(info.into())
}

pub const PATH_GET_SYSTEM_INFO: &str = "/manager_api/system_info";

/// Get system info about current operating system, hardware and software.
///
/// Returns system info related to current manager instance.
#[utoipa::path(
    get,
    path = "/manager_api/system_info",
    responses(
        (status = 200, description = "System info", body = SystemInfo),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn get_system_info<S: GetConfig>(
    ConnectInfo(client): ConnectInfo<SocketAddr>,
    state: S,
) -> Result<Json<SystemInfo>, StatusCode> {
    info!("Get current system info received from {}.", client,);

    let info = SystemInfoGetter::system_info(state.config()).await?;
    Ok(info.into())
}

pub const PATH_GET_SYSTEM_INFO_ALL: &str = "/manager_api/system_info_all";

/// Get system info about current operating system, hardware and software.
///
/// Returns system info related to current manager instance and ones
/// defined in config file.
#[utoipa::path(
    get,
    path = "/manager_api/system_info_all",
    responses(
        (status = 200, description = "Get all system infos available", body = SystemInfoList),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn get_system_info_all<S: GetConfig + GetApiManager>(
    ConnectInfo(client): ConnectInfo<SocketAddr>,
    state: S,
) -> Result<Json<SystemInfoList>, StatusCode> {
    info!("Get all system infos received from {}.", client,);

    let info = SystemInfoGetter::system_info_all(state.config(), &state.api_manager()).await?;
    Ok(info.into())
}
