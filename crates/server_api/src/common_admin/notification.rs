use axum::{Extension, extract::State};
use model::{AccountIdInternal, AdminNotification, PendingNotificationFlags, Permissions};
use server_data::{
    app::EventManagerProvider, read::GetReadCommandsCommon, write::GetWriteCommandsCommon,
};
use server_state::{app::AdminNotificationProvider, db_write_multiple};
use simple_backend::create_counters;

use crate::{
    S,
    app::{ReadData, WriteData},
    create_open_api_router,
    utils::{Json, StatusCode},
};

const PATH_GET_ADMIN_NOTIFICATION_SUBSCRIPTIONS: &str =
    "/common_api/admin_notification_subscriptions";

/// Get admin notification subscriptions.
///
/// # Access
/// Requires [Permissions::admin_subscribe_admin_notifications].
#[utoipa::path(
    get,
    path = PATH_GET_ADMIN_NOTIFICATION_SUBSCRIPTIONS,
    responses(
        (status = 200, description = "Successful.", body = AdminNotification),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_admin_notification_subscriptions(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<AdminNotification>, StatusCode> {
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

const PATH_POST_ADMIN_NOTIFICATION_SUBSCRIPTIONS: &str =
    "/common_api/admin_notification_subscriptions";

/// Save admin notification subscriptions.
///
/// # Access
/// Requires [Permissions::admin_subscribe_admin_notifications].
#[utoipa::path(
    post,
    path = PATH_POST_ADMIN_NOTIFICATION_SUBSCRIPTIONS,
    request_body = AdminNotification,
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
    Json(subscriptions): Json<AdminNotification>,
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

const PATH_POST_GET_ADMIN_NOTIFICATION: &str = "/common_api/admin_notification";

/// Get admin notification data.
///
/// Getting notification data is required when notification event is received
/// from WebSocket. This prevents resending the notification as push
/// notification when WebSocket connection closes.
///
/// # Access
/// Requires [Permissions::admin_subscribe_admin_notifications].
#[utoipa::path(
    post,
    path = PATH_POST_GET_ADMIN_NOTIFICATION,
    responses(
        (status = 200, description = "Successfull.", body = AdminNotification),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_admin_notification(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<AdminNotification>, StatusCode> {
    COMMON_ADMIN.post_get_admin_notification.incr();

    if api_caller_permissions.admin_subscribe_admin_notifications {
        state
            .event_manager()
            .remove_specific_pending_notification_flags_from_cache(
                api_caller_account_id,
                PendingNotificationFlags::ADMIN_NOTIFICATION,
            )
            .await;

        let data = state
            .admin_notification()
            .get_notification_state(api_caller_account_id)
            .await
            .unwrap_or_default();

        state
            .admin_notification()
            .reset_notification_state(api_caller_account_id)
            .await;

        Ok(data.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

create_open_api_router!(fn router_notification, get_admin_notification_subscriptions, post_admin_notification_subscriptions, post_get_admin_notification,);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_NOTIFICATION_COUNTERS_LIST,
    get_admin_notification_subscriptions,
    post_admin_notification_subscriptions,
    post_get_admin_notification,
);
