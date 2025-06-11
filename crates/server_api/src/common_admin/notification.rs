use axum::{extract::State, Extension};
use model::{AccountIdInternal, AdminNotificationSubscriptions, Permissions};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use server_state::db_write_multiple;
use simple_backend::create_counters;

use crate::{
    app::{ReadData, WriteData},
    create_open_api_router,
    utils::{Json, StatusCode},
    S,
};

const PATH_GET_ADMIN_NOTIFICATION_SUBSCRIPTIONS: &str = "/common_api/admin_notification_subscriptions";

/// Get admin notification subscriptions.
///
/// # Access
/// Requires [Permissions::admin_subscribe_admin_notifications].
#[utoipa::path(
    get,
    path = PATH_GET_ADMIN_NOTIFICATION_SUBSCRIPTIONS,
    responses(
        (status = 200, description = "Successful.", body = AdminNotificationSubscriptions),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_admin_notification_subscriptions(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<AdminNotificationSubscriptions>, StatusCode> {
    COMMON_ADMIN.get_admin_notification_subscriptions.incr();

    if api_caller_permissions.admin_server_maintenance_view_backend_config {
        let subscriptions = state
            .read()
            .common_admin()
            .notification()
            .admin_notification_subscriptions(api_caller_account_id)
            .await?;
        Ok(subscriptions.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_ADMIN_NOTIFICATION_SUBSCRIPTIONS: &str = "/common_api/admin_notification_subscriptions";

/// Save admin notification subscriptions.
///
/// # Access
/// Requires [Permissions::admin_subscribe_admin_notifications].
#[utoipa::path(
    post,
    path = PATH_POST_ADMIN_NOTIFICATION_SUBSCRIPTIONS,
    request_body = AdminNotificationSubscriptions,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_admin_notification_subscriptions(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(subscriptions): Json<AdminNotificationSubscriptions>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_admin_notification_subscriptions.incr();

    if api_caller_permissions.admin_subscribe_admin_notifications {
        db_write_multiple!(state, move |cmds| {
            cmds.common_admin()
                .notification()
                .set_admin_notification_subscriptions(api_caller_account_id, subscriptions)
                .await?;
            Ok(())
        })?;

        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

create_open_api_router!(fn router_notification, get_admin_notification_subscriptions, post_admin_notification_subscriptions,);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_NOTIFICATION_COUNTERS_LIST,
    get_admin_notification_subscriptions,
    post_admin_notification_subscriptions,
);
