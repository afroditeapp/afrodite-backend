use axum::{extract::State, Extension, Router};
use model::{AccountIdInternal, FcmDeviceToken, PendingNotification};
use simple_backend::{create_counters};

use super::super::utils::{Json, StatusCode};
use crate::{app::{GetAccounts, WriteData}, db_write};

// TODO(prod): Logout route should remove the device token
// TODO(prod): Connecting with websocket should reset the pending notification

pub const PATH_POST_SET_DEVICE_TOKEN: &str = "/chat_api/set_device_token";

#[utoipa::path(
    post,
    path = PATH_POST_SET_DEVICE_TOKEN,
    request_body(content = FcmDeviceToken),
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_set_device_token<S: WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(device_token): Json<FcmDeviceToken>,
) -> Result<(), StatusCode> {
    CHAT.post_set_device_token.incr();

    db_write!(state, move |cmds| {
        cmds.chat().push_notifications().set_device_token(id, device_token)
    })?;

    Ok(())
}

pub const PATH_POST_GET_PENDING_NOTIFICATION: &str = "/chat_api/get_pending_notification";

/// Get pending notification and reset pending notification.
///
/// Requesting this route is always valid to avoid figuring out device
/// token values more easily.
#[utoipa::path(
    post,
    path = PATH_POST_GET_PENDING_NOTIFICATION,
    request_body(content = FcmDeviceToken),
    responses(
        (status = 200, description = "Success", body = PendingNotification),
    ),
    security(), // This is public route handler
)]
pub async fn post_get_pending_notification<S: GetAccounts + WriteData>(
    State(state): State<S>,
    Json(token): Json<FcmDeviceToken>,
) -> Json<PendingNotification> {
    CHAT.post_get_pending_notification.incr();

    let flags: PendingNotification = db_write!(state, move |cmds| {
        cmds.chat().push_notifications().get_and_reset_pending_notification_with_device_token(token)
    })
        .unwrap_or_default();

    flags.into()
}

pub fn push_notification_router_private(s: crate::app::S) -> Router {
    use axum::routing::post;

    use crate::app::S;

    Router::new()
        .route(PATH_POST_SET_DEVICE_TOKEN, post(post_set_device_token::<S>))
        .with_state(s)
}

pub fn push_notification_router_public(s: crate::app::S) -> Router {
    use axum::routing::post;

    use crate::app::S;

    Router::new()
        .route(PATH_POST_GET_PENDING_NOTIFICATION, post(post_get_pending_notification::<S>))
        .with_state(s)
}

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_PUSH_NOTIFICATION_COUNTERS_LIST,
    post_set_device_token,
    post_get_pending_notification,
);
