use axum::{
    extract::{Query, State},
    Extension,
};
use manager_model::{
    ManagerInstanceName, ManagerInstanceNameList, SoftwareInfo, SoftwareUpdateStatus, SystemInfo
};
use model::Permissions;
use simple_backend::{app::GetManagerApi, create_counters};
use manager_api::RequestSenderCmds;

use crate::{
    create_open_api_router,
    utils::{Json, StatusCode},
    S,
};

const PATH_GET_MANAGER_INSTANCE_NAMES: &str = "/common_api/manager_instance_names";

/// Get available manager instances.
///
/// # Access
/// * Permission [model::Permissions::admin_server_maintenance_view_info]
/// * Permission [model::Permissions::admin_server_maintenance_update_software]
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

    if api_caller_permissions.admin_server_maintenance_view_info ||
        api_caller_permissions.admin_server_maintenance_update_software
    {
        let info = state.manager_request().await?.get_available_instances().await?;
        Ok(info.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_GET_SYSTEM_INFO: &str = "/common_api/system_info";

/// Get system information from manager instance.
///
/// # Access
/// * Permission [model::Permissions::admin_server_maintenance_view_info]
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
///
/// # Access
/// * Permission [model::Permissions::admin_server_maintenance_view_info]
#[utoipa::path(
    get,
    path = PATH_GET_SOFTWARE_INFO,
    params(ManagerInstanceName),
    responses(
        (status = 200, description = "Get was successfull.", body = SoftwareUpdateStatus),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_software_update_status(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Query(manager): Query<ManagerInstanceName>,
) -> Result<Json<SoftwareUpdateStatus>, StatusCode> {
    COMMON_ADMIN.get_software_update_status.incr();

    if api_caller_permissions.admin_server_maintenance_view_info {
        let info = state.manager_request_to(manager).await?.get_software_update_status().await?;
        Ok(info.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_TRIGGER_SOFTWARE_UPDATE_DOWNLOAD: &str = "/common_api/trigger_software_download";

/// Trigger software update download.
///
/// # Access
/// * Permission [model::Permissions::admin_server_maintenance_update_software]
#[utoipa::path(
    post,
    path = PATH_POST_TRIGGER_SOFTWARE_UPDATE_DOWNLOAD,
    params(ManagerInstanceName),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_trigger_software_update_download(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Query(manager): Query<ManagerInstanceName>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_trigger_software_update_download.incr();

    if api_caller_permissions.admin_server_maintenance_update_software {
        state.manager_request_to(manager).await?.trigger_software_update_download().await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_TRIGGER_SOFTWARE_UPDATE_INSTALL: &str = "/common_api/trigger_software_install";

/// Trigger software update install.
///
/// # Access
/// * Permission [model::Permissions::admin_server_maintenance_update_software]
#[utoipa::path(
    post,
    path = PATH_POST_TRIGGER_SOFTWARE_UPDATE_INSTALL,
    params(ManagerInstanceName, SoftwareInfo),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_trigger_software_update_install(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Query(manager): Query<ManagerInstanceName>,
    Query(info): Query<SoftwareInfo>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_trigger_software_update_install.incr();

    if api_caller_permissions.admin_server_maintenance_update_software {
        state.manager_request_to(manager).await?.trigger_software_update_install(info).await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_TRIGGER_BACKEND_DATA_RESET: &str = "/common_api/trigger_backend_data_reset";

/// Trigger backend data reset which also restarts the backend.
///
/// # Access
/// * Permission [model::Permissions::admin_server_maintenance_reset_data]
#[utoipa::path(
    post,
    path = PATH_POST_TRIGGER_BACKEND_DATA_RESET,
    params(ManagerInstanceName),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_trigger_backend_data_reset(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Query(manager): Query<ManagerInstanceName>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_trigger_backend_data_reset.incr();

    if api_caller_permissions.admin_server_maintenance_reset_data {
        state.manager_request_to(manager).await?.trigger_backend_data_reset().await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_TRIGGER_BACKEND_RESTART: &str = "/common_api/trigger_backend_restart";

/// Trigger backend restart.
///
/// # Access
/// * Permission [model::Permissions::admin_server_maintenance_reboot_backend]
#[utoipa::path(
    post,
    path = PATH_POST_TRIGGER_BACKEND_RESTART,
    params(ManagerInstanceName),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_trigger_backend_restart(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Query(manager): Query<ManagerInstanceName>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_trigger_backend_restart.incr();

    if api_caller_permissions.admin_server_maintenance_reboot_backend {
        state.manager_request_to(manager).await?.trigger_backend_restart().await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

create_open_api_router!(
        fn router_manager,
        get_manager_instance_names,
        get_system_info,
        get_software_update_status,
        post_trigger_software_update_download,
        post_trigger_software_update_install,
        post_trigger_backend_data_reset,
        post_trigger_backend_restart,
);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_MANAGER_COUNTERS_LIST,
    get_manager_instance_names,
    get_system_info,
    get_software_update_status,
    get_latest_build_info,
    post_request_build_software,
    post_trigger_software_update_download,
    post_trigger_software_update_install,
    post_trigger_backend_data_reset,
    post_trigger_backend_restart,
);
