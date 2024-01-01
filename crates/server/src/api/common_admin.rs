//! Common routes related to admin features

use axum::{extract::{Query, State}, Extension};
use manager_model::{
    BuildInfo, RebootQueryParam, ResetDataQueryParam, SoftwareInfo, SoftwareOptionsQueryParam,
    SystemInfoList,
};
use model::{AccountIdInternal, BackendConfig, Capabilities};
use simple_backend::{app::{GetManagerApi, PerfCounterDataProvider}, create_counters};
use simple_backend_model::{PerfHistoryQuery, PerfHistoryQueryResult};
use tracing::info;

use crate::{
    api::utils::{Json, StatusCode},
    app::{ReadData, ReadDynamicConfig, WriteDynamicConfig},
};

pub const PATH_GET_SYSTEM_INFO: &str = "/common_api/system_info";

/// Get system information from manager instance.
#[utoipa::path(
    get,
    path = "/common_api/system_info",
    responses(
        (status = 200, description = "Get was successfull.", body = SystemInfoList),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_system_info<S: GetManagerApi>(
    State(state): State<S>,
    Extension(api_caller_capabilities): Extension<Capabilities>,
) -> Result<Json<SystemInfoList>, StatusCode> {
    COMMON_ADMIN.get_system_info.incr();

    if api_caller_capabilities.admin_server_maintenance_view_info {
        let info = state.manager_api().system_info().await?;
        Ok(info.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_GET_SOFTWARE_INFO: &str = "/common_api/software_info";

/// Get software version information from manager instance.
#[utoipa::path(
    get,
    path = "/common_api/software_info",
    responses(
        (status = 200, description = "Get was successfull.", body = SoftwareInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_software_info<S: GetManagerApi>(
    State(state): State<S>,
    Extension(api_caller_capabilities): Extension<Capabilities>,
) -> Result<Json<SoftwareInfo>, StatusCode> {
    COMMON_ADMIN.get_software_info.incr();

    if api_caller_capabilities.admin_server_maintenance_view_info {
        let info = state.manager_api().software_info().await?;
        Ok(info.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_GET_LATEST_BUILD_INFO: &str = "/common_api/get_latest_build_info";

/// Get latest software build information available for update from manager
/// instance.
#[utoipa::path(
    get,
    path = "/common_api/get_latest_build_info",
    params(SoftwareOptionsQueryParam),
    responses(
        (status = 200, description = "Get was successfull.", body = BuildInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_latest_build_info<S: GetManagerApi>(
    State(state): State<S>,
    Query(software): Query<SoftwareOptionsQueryParam>,
    Extension(api_caller_capabilities): Extension<Capabilities>,
) -> Result<Json<BuildInfo>, StatusCode> {
    COMMON_ADMIN.get_latest_build_info.incr();

    if api_caller_capabilities.admin_server_maintenance_view_info {
        let info = state
            .manager_api()
            .get_latest_build_info(software.software_options)
            .await?;
        Ok(info.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_POST_REQUEST_BUILD_SOFTWARE: &str = "/common_api/request_build_software";

/// Request building new software from manager instance.
#[utoipa::path(
    post,
    path = "/common_api/request_build_software",
    params(SoftwareOptionsQueryParam),
    responses(
        (status = 200, description = "Request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_request_build_software<S: GetManagerApi>(
    State(state): State<S>,
    Query(software): Query<SoftwareOptionsQueryParam>,
    Extension(api_caller_capabilities): Extension<Capabilities>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_request_build_software.incr();

    if api_caller_capabilities
        .admin_server_maintenance_update_software
    {
        state
            .manager_api()
            .request_build_software_from_build_server(software.software_options)
            .await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_POST_REQUEST_UPDATE_SOFTWARE: &str = "/common_api/request_update_software";

/// Request updating new software from manager instance.
///
/// Reboot query parameter will force reboot of the server after update.
/// If it is off, the server will be rebooted when the usual reboot check
/// is done.
///
/// Reset data query parameter will reset data like defined in current
/// app-manager version. If this is true then specific capability is needed
/// for completing this request.
///
/// # Capablities
/// Requires admin_server_maintenance_update_software. Also requires
/// admin_server_maintenance_reset_data if reset_data is true.
#[utoipa::path(
    post,
    path = "/common_api/request_update_software",
    params(SoftwareOptionsQueryParam, RebootQueryParam, ResetDataQueryParam),
    responses(
        (status = 200, description = "Request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_request_update_software<S: GetManagerApi>(
    State(state): State<S>,
    Query(software): Query<SoftwareOptionsQueryParam>,
    Query(reboot): Query<RebootQueryParam>,
    Query(reset_data): Query<ResetDataQueryParam>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_capabilities): Extension<Capabilities>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_request_update_software.incr();

    if reset_data.reset_data && !api_caller_capabilities.admin_server_maintenance_reset_data {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if api_caller_capabilities
        .admin_server_maintenance_update_software
    {
        info!(
            "Requesting update software, account: {}, software: {:?}, reboot: {}, reset_data: {},",
            api_caller_account_id.as_uuid(),
            software.software_options,
            reboot.reboot,
            reset_data.reset_data,
        );
        state
            .manager_api()
            .request_update_software(software.software_options, reboot.reboot, reset_data)
            .await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_POST_REQUEST_RESTART_OR_RESET_BACKEND: &str =
    "/common_api/request_restart_or_reset_backend";

/// Request restarting or reseting backend through app-manager instance.
///
/// # Capabilities
/// Requires admin_server_maintenance_restart_backend. Also requires
/// admin_server_maintenance_reset_data if reset_data is true.
#[utoipa::path(
    post,
    path = "/common_api/request_restart_or_reset_backend",
    params(ResetDataQueryParam),
    responses(
        (status = 200, description = "Request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_request_restart_or_reset_backend<S: GetManagerApi>(
    State(state): State<S>,
    Query(reset_data): Query<ResetDataQueryParam>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_capabilities): Extension<Capabilities>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_request_restart_or_reset_backend.incr();

    if reset_data.reset_data && !api_caller_capabilities.admin_server_maintenance_reset_data {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if api_caller_capabilities
        .admin_server_maintenance_update_software
    {
        info!(
            "Requesting reset or restart backend, account: {}, reset_data: {}",
            api_caller_account_id.as_uuid(),
            reset_data.reset_data,
        );
        state
            .manager_api()
            .request_restart_or_reset_backend(reset_data)
            .await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_GET_BACKEND_CONFIG: &str = "/common_api/backend_config";

/// Get dynamic backend config.
///
/// # Capabilities
/// Requires admin_server_maintenance_view_backend_settings.
#[utoipa::path(
    get,
    path = "/common_api/backend_config",
    params(),
    responses(
        (status = 200, description = "Get was successfull.", body = BackendConfig),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_backend_config<S: ReadDynamicConfig>(
    State(state): State<S>,
    Extension(api_caller_capabilities): Extension<Capabilities>,
) -> Result<Json<BackendConfig>, StatusCode> {
    COMMON_ADMIN.get_backend_config.incr();

    if api_caller_capabilities
        .admin_server_maintenance_view_backend_config
    {
        let config = state.read_config().await?;
        Ok(config.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_POST_BACKEND_CONFIG: &str = "/common_api/backend_config";

/// Save dynamic backend config.
///
/// # Capabilities
/// Requires admin_server_maintenance_save_backend_settings.
#[utoipa::path(
    post,
    path = "/common_api/backend_config",
    params(),
    request_body(content = BackendConfig),
    responses(
        (status = 200, description = "Save was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_backend_config<S: ReadData + WriteDynamicConfig>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_capabilities): Extension<Capabilities>,
    Json(backend_config): Json<BackendConfig>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_backend_config.incr();

    if api_caller_capabilities
        .admin_server_maintenance_save_backend_config
    {
        info!(
            "Saving dynamic backend config, account: {}, settings: {:#?}",
            api_caller_account_id.as_uuid(),
            backend_config
        );
        state.write_config(backend_config).await?;

        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub const PATH_GET_PERF_DATA: &str = "/common_api/perf_data";

/// Get performance data
///
/// # Capabilities
/// Requires admin_server_maintenance_view_info.
#[utoipa::path(
    get,
    path = "/common_api/perf_data",
    params(PerfHistoryQuery),
    responses(
        (status = 200, description = "Get was successfull.", body = PerfHistoryQueryResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_perf_data<S: PerfCounterDataProvider>(
    State(state): State<S>,
    Extension(api_caller_capabilities): Extension<Capabilities>,
    Query(_query): Query<PerfHistoryQuery>,
) -> Result<Json<PerfHistoryQueryResult>, StatusCode> {
    COMMON_ADMIN.get_perf_data.incr();
    if api_caller_capabilities.admin_server_maintenance_view_info {
        let data = state.perf_counter_data().get_history().await;
        Ok(data.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_COUNTERS_LIST,
    get_system_info,
    get_software_info,
    get_latest_build_info,
    post_request_build_software,
    post_request_update_software,
    post_request_restart_or_reset_backend,
    get_backend_config,
    post_backend_config,
    get_perf_data,
);
