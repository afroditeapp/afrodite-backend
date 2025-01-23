use axum::{
    extract::{Query, State},
    Extension,
};
use manager_model::{
    ManagerInstanceName, ManagerInstanceNameList, ManualTaskType, NotifyBackend, ScheduledTaskStatus, ScheduledTaskType, ScheduledTaskTypeValue, SoftwareInfo, SoftwareUpdateStatus, SoftwareUpdateTaskType, SystemInfo
};
use model::Permissions;
use simple_backend::{app::GetManagerApi, create_counters};
use manager_api::RequestSenderCmds;

use crate::{
    create_open_api_router,
    utils::{Json, StatusCode},
    S,
};

// TODO(prod): Rename admin_server_maintenance_reset_data to admin_server_maintenance_data_reset?
// TODO(prod): Rename admin_server_maintenance_reboot_backend to
// admin_server_maintenance_backend_restart or admin_server_maintenance_reboot_and_restart?

const PATH_GET_MANAGER_INSTANCE_NAMES: &str = "/common_api/manager_instance_names";

/// Get available manager instances.
///
/// # Access
/// * Permission [model::Permissions::admin_server_maintenance_view_info]
/// * Permission [model::Permissions::admin_server_maintenance_update_software]
/// * Permission [model::Permissions::admin_server_maintenance_reset_data]
/// * Permission [model::Permissions::admin_server_maintenance_reboot_backend]
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
        api_caller_permissions.admin_server_maintenance_update_software ||
        api_caller_permissions.admin_server_maintenance_reset_data ||
        api_caller_permissions.admin_server_maintenance_reboot_backend
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
        state.manager_request_to(manager)
            .await?
            .trigger_software_update_task(SoftwareUpdateTaskType::Download)
            .await?;
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
        state.manager_request_to(manager)
            .await?
            .trigger_software_update_task(SoftwareUpdateTaskType::Install(info))
            .await?;
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
        state.manager_request_to(manager)
            .await?
            .trigger_manual_task(ManualTaskType::BackendDataReset)
            .await?;
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
        state.manager_request_to(manager)
            .await?
            .trigger_manual_task(ManualTaskType::BackendRestart)
            .await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_TRIGGER_SYSTEM_REBOOT: &str = "/common_api/trigger_system_reboot";

/// Trigger system reboot.
///
/// # Access
/// * Permission [model::Permissions::admin_server_maintenance_reboot_backend]
#[utoipa::path(
    post,
    path = PATH_POST_TRIGGER_SYSTEM_REBOOT,
    params(ManagerInstanceName),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_trigger_system_reboot(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Query(manager): Query<ManagerInstanceName>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_trigger_system_reboot.incr();

    if api_caller_permissions.admin_server_maintenance_reboot_backend {
        state.manager_request_to(manager)
            .await?
            .trigger_manual_task(ManualTaskType::SystemReboot)
            .await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_GET_SCHEDULED_TASKS_STATUS: &str = "/common_api/scheduled_tasks_status";

/// Get scheduled tasks status from manager instance.
///
/// # Access
/// * Permission [model::Permissions::admin_server_maintenance_reboot_backend]
#[utoipa::path(
    get,
    path = PATH_GET_SCHEDULED_TASKS_STATUS,
    params(ManagerInstanceName),
    responses(
        (status = 200, description = "Successful.", body = ScheduledTaskStatus),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_scheduled_tasks_status(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Query(manager): Query<ManagerInstanceName>,
) -> Result<Json<ScheduledTaskStatus>, StatusCode> {
    COMMON_ADMIN.get_software_update_status.incr();

    if api_caller_permissions.admin_server_maintenance_reboot_backend {
        let info = state.manager_request_to(manager).await?.get_scheduled_tasks_status().await?;
        Ok(info.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_SCHEDULE_TASK: &str = "/common_api/schedule_task";

/// Schedule task.
///
/// # Access
/// * Permission [model::Permissions::admin_server_maintenance_reboot_backend]
#[utoipa::path(
    post,
    path = PATH_POST_SCHEDULE_TASK,
    params(ManagerInstanceName, ScheduledTaskTypeValue, NotifyBackend),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_schedule_task(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Query(manager): Query<ManagerInstanceName>,
    Query(task): Query<ScheduledTaskTypeValue>,
    Query(notify_backend): Query<NotifyBackend>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_schedule_task.incr();

    if api_caller_permissions.admin_server_maintenance_reboot_backend {
        state.manager_request_to(manager)
            .await?
            .schedule_task(task.scheduled_task_type, notify_backend)
            .await?;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_UNSCHEDULE_TASK: &str = "/common_api/unschedule_task";

/// Unschedule task.
///
/// # Access
/// * Permission [model::Permissions::admin_server_maintenance_reboot_backend]
#[utoipa::path(
    post,
    path = PATH_POST_UNSCHEDULE_TASK,
    params(ManagerInstanceName, ScheduledTaskTypeValue),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_unschedule_task(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Query(manager): Query<ManagerInstanceName>,
    Query(task): Query<ScheduledTaskTypeValue>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_unschedule_task.incr();

    if api_caller_permissions.admin_server_maintenance_reboot_backend {
        state.manager_request_to(manager)
            .await?
            .unschedule_task(task.scheduled_task_type)
            .await?;
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
        post_trigger_system_reboot,
        get_scheduled_tasks_status,
        post_schedule_task,
        post_unschedule_task,
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
    post_trigger_system_reboot,
    get_scheduled_tasks_status,
    post_schedule_task,
    post_unschedule_task,
);
