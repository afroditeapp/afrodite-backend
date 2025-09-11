use axum::{Extension, extract::State};
use model::{EventToClientInternal, Permissions, ScheduledMaintenanceStatus};
use server_data::app::EventManagerProvider;
use simple_backend::{app::GetManagerApi, create_counters};

use crate::{
    S, create_open_api_router,
    utils::{Json, StatusCode},
};

const PATH_GET_MAINTENANCE_NOTIFICATION: &str = "/common_api/maintenance_notification";

/// Get maintenance notification.
///
/// # Permissions
/// Requires admin_server_maintenance_edit_notification.
#[utoipa::path(
    get,
    path = PATH_GET_MAINTENANCE_NOTIFICATION,
    responses(
        (status = 200, description = "Successful.", body = ScheduledMaintenanceStatus),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_maintenance_notification(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<ScheduledMaintenanceStatus>, StatusCode> {
    COMMON_ADMIN.get_maintenance_notification.incr();

    if api_caller_permissions.admin_server_maintenance_edit_notification {
        Ok(state.manager_api_client().maintenance_status().await.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_EDIT_MAINTENANCE_NOTIFICATION: &str = "/common_api/edit_maintenance_notification";

/// Edit maintenance notification
///
/// # Permissions
/// Requires admin_server_maintenance_edit_notification.
#[utoipa::path(
    post,
    path = PATH_POST_EDIT_MAINTENANCE_NOTIFICATION,
    request_body = ScheduledMaintenanceStatus,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_edit_maintenance_notification(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(status): Json<ScheduledMaintenanceStatus>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_edit_maintenance_notification.incr();

    if api_caller_permissions.admin_server_maintenance_edit_notification {
        state
            .manager_api_client()
            .set_maintenance_status(status.clone())
            .await;
        state
            .event_manager()
            .send_connected_event_to_logged_in_clients(
                EventToClientInternal::ScheduledMaintenanceStatus(status),
            )
            .await;
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

create_open_api_router!(
    fn router_maintenance,
    get_maintenance_notification,
    post_edit_maintenance_notification,
);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_MAINTENANCE_COUNTERS_LIST,
    get_maintenance_notification,
    post_edit_maintenance_notification,
);
