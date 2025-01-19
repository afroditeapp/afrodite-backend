use axum::{
    extract::{Query, State},
    Extension,
};
use manager_model::{
    BuildInfo, ManagerInstanceName, ManagerInstanceNameList, RebootQueryParam, ResetDataQueryParam, SoftwareInfo, SoftwareOptionsQueryParam, SystemInfo
};
use model::{AccountIdInternal, Permissions};
use simple_backend::{app::GetManagerApi, create_counters};
use tracing::info;
use manager_api::RequestSenderCmds;

use crate::{
    create_open_api_router,
    utils::{Json, StatusCode},
    S,
};

const PATH_GET_MANAGER_INSTANCE_NAMES: &str = "/common_api/manager_instance_names";

#[utoipa::path(
    get,
    path = PATH_GET_MANAGER_INSTANCE_NAMES,
    responses(
        (status = 200, description = "Successful.", body = ManagerInstanceNameList),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_manager_instance_names(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<ManagerInstanceNameList>, StatusCode> {
    COMMON_ADMIN.get_manager_instance_names.incr();

    if api_caller_permissions.admin_server_maintenance_view_info {
        let info = state.manager_request().await?.get_available_instances().await?;
        Ok(info.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_GET_SYSTEM_INFO: &str = "/common_api/system_info";

/// Get system information from manager instance.
#[utoipa::path(
    get,
    path = PATH_GET_SYSTEM_INFO,
    params(ManagerInstanceName),
    responses(
        (status = 200, description = "Successful.", body = SystemInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_system_info(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Query(manager): Query<ManagerInstanceName>,
) -> Result<Json<SystemInfo>, StatusCode> {
    COMMON_ADMIN.get_system_info.incr();

    if api_caller_permissions.admin_server_maintenance_view_info {
        let info = state.manager_request_to(manager).await?.get_system_info().await?;
        Ok(info.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_GET_SOFTWARE_INFO: &str = "/common_api/software_info";

/// Get software version information from manager instance.
#[utoipa::path(
    get,
    path = PATH_GET_SOFTWARE_INFO,
    responses(
        (status = 200, description = "Get was successfull.", body = SoftwareInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_software_info(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<SoftwareInfo>, StatusCode> {
    COMMON_ADMIN.get_software_info.incr();

    if api_caller_permissions.admin_server_maintenance_view_info {
        // let info = state.manager_api().software_info().await?;
        // Ok(info.into())
        Err(StatusCode::UNAUTHORIZED)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_GET_LATEST_BUILD_INFO: &str = "/common_api/get_latest_build_info";

/// Get latest software build information available for update from manager
/// instance.
#[utoipa::path(
    get,
    path = PATH_GET_LATEST_BUILD_INFO,
    params(SoftwareOptionsQueryParam),
    responses(
        (status = 200, description = "Get was successfull.", body = BuildInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_latest_build_info(
    State(state): State<S>,
    Query(software): Query<SoftwareOptionsQueryParam>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<BuildInfo>, StatusCode> {
    COMMON_ADMIN.get_latest_build_info.incr();

    if api_caller_permissions.admin_server_maintenance_view_info {
        // let info = state
        //     .manager_api()
        //     .get_latest_build_info(software.software_options)
        //     .await?;
        // Ok(info.into())
        Err(StatusCode::UNAUTHORIZED)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_REQUEST_UPDATE_SOFTWARE: &str = "/common_api/request_update_software";

/// Request updating new software from manager instance.
///
/// Reboot query parameter will force reboot of the server after update.
/// If it is off, the server will be rebooted when the usual reboot check
/// is done.
///
/// Reset data query parameter will reset data like defined in current
/// app-manager version. If this is true then specific permission is needed
/// for completing this request.
///
/// # Permissions
/// Requires admin_server_maintenance_update_software. Also requires
/// admin_server_maintenance_reset_data if reset_data is true.
#[utoipa::path(
    post,
    path = PATH_POST_REQUEST_UPDATE_SOFTWARE,
    params(SoftwareOptionsQueryParam, RebootQueryParam, ResetDataQueryParam),
    responses(
        (status = 200, description = "Request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_request_update_software(
    State(state): State<S>,
    Query(software): Query<SoftwareOptionsQueryParam>,
    Query(reboot): Query<RebootQueryParam>,
    Query(reset_data): Query<ResetDataQueryParam>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_request_update_software.incr();

    if reset_data.reset_data && !api_caller_permissions.admin_server_maintenance_reset_data {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if api_caller_permissions.admin_server_maintenance_update_software {
        info!(
            "Requesting update software, account: {}, software: {:?}, reboot: {}, reset_data: {},",
            api_caller_account_id.as_id(),
            software.software_options,
            reboot.reboot,
            reset_data.reset_data,
        );
        // state
        //     .manager_api()
        //     .request_update_software(software.software_options, reboot.reboot, reset_data)
        //     .await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_REQUEST_RESTART_OR_RESET_BACKEND: &str =
    "/common_api/request_restart_or_reset_backend";

/// Request restarting or reseting backend through app-manager instance.
///
/// # Permissions
/// Requires admin_server_maintenance_restart_backend. Also requires
/// admin_server_maintenance_reset_data if reset_data is true.
#[utoipa::path(
    post,
    path = PATH_POST_REQUEST_RESTART_OR_RESET_BACKEND,
    params(ResetDataQueryParam),
    responses(
        (status = 200, description = "Request was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_request_restart_or_reset_backend(
    State(state): State<S>,
    Query(reset_data): Query<ResetDataQueryParam>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_request_restart_or_reset_backend.incr();

    if reset_data.reset_data && !api_caller_permissions.admin_server_maintenance_reset_data {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if api_caller_permissions.admin_server_maintenance_update_software {
        info!(
            "Requesting reset or restart backend, account: {}, reset_data: {}",
            api_caller_account_id.as_id(),
            reset_data.reset_data,
        );
        // state
        //     .manager_api()
        //     .request_restart_or_reset_backend(reset_data)
        //     .await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

create_open_api_router!(
        fn router_manager,
        get_manager_instance_names,
        get_system_info,
        get_software_info,
        get_latest_build_info,
        post_request_update_software,
        post_request_restart_or_reset_backend,
);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_MANAGER_COUNTERS_LIST,
    get_manager_instance_names,
    get_system_info,
    get_software_info,
    get_latest_build_info,
    post_request_build_software,
    post_request_update_software,
    post_request_restart_or_reset_backend,
);
