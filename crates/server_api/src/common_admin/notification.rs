use axum::{Extension, extract::State};
use model::{
    AccountIdInternal, AdminNotification, AdminNotificationSettings, PendingNotificationFlags,
    Permissions,
};
use server_data::{
    app::EventManagerProvider, read::GetReadCommandsCommon, write::GetWriteCommandsCommon,
};
use server_state::{app::AdminNotificationProvider, db_write};
use simple_backend::create_counters;

use crate::{
    S,
    app::{ReadData, WriteData},
    create_open_api_router,
    utils::{Json, StatusCode},
};

const PATH_GET_ADMIN_NOTIFICATION_SETTINGS: &str = "/common_api/admin_notification_settings";

/// Get admin notification settings.
///
/// # Access
/// Requires [Permissions::admin_subscribe_admin_notifications].
#[utoipa::path(
    get,
    path = PATH_GET_ADMIN_NOTIFICATION_SETTINGS,
    responses(
        (status = 200, description = "Successful.", body = AdminNotificationSettings),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_admin_notification_settings(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<AdminNotificationSettings>, StatusCode> {
    COMMON_ADMIN.get_admin_notification_settings.incr();

    if api_caller_permissions.admin_subscribe_admin_notifications {
        let settings = state
            .read()
            .common_admin()
            .notification()
            .admin_notification_settings(api_caller_account_id)
            .await?;
        Ok(settings.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_ADMIN_NOTIFICATION_SETTINGS: &str = "/common_api/admin_notification_settings";

/// Save admin notification settings.
///
/// # Access
/// Requires [Permissions::admin_subscribe_admin_notifications].
#[utoipa::path(
    post,
    path = PATH_POST_ADMIN_NOTIFICATION_SETTINGS,
    request_body = AdminNotificationSettings,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_admin_notification_settings(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(settings): Json<AdminNotificationSettings>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_admin_notification_settings.incr();

    if api_caller_permissions.admin_subscribe_admin_notifications {
        db_write!(state, move |cmds| {
            cmds.common_admin()
                .notification()
                .set_admin_notification_settings(api_caller_account_id, settings)
                .await?;
            Ok(())
        })?;

        state.admin_notification().refresh_start_time_waiter().await;

        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

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

    if api_caller_permissions.admin_subscribe_admin_notifications {
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
        db_write!(state, move |cmds| {
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
            .write()
            .mark_notification_received_and_return_it(api_caller_account_id)
            .await;

        Ok(data.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

create_open_api_router!(
    fn router_notification,
    get_admin_notification_settings,
    post_admin_notification_settings,
    get_admin_notification_subscriptions,
    post_admin_notification_subscriptions,
    post_get_admin_notification,
);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_NOTIFICATION_COUNTERS_LIST,
    get_admin_notification_settings,
    post_admin_notification_settings,
    get_admin_notification_subscriptions,
    post_admin_notification_subscriptions,
    post_get_admin_notification,
);
