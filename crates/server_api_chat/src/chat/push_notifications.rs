use axum::{extract::State, Extension, Router};
use model::{AccountIdInternal, FcmDeviceToken, PendingNotificationFlags, PendingNotificationToken, PendingNotificationWithData};
use server_api::app::ReadData;
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::{
    app::{GetAccounts, StateBase, WriteData},
    db_write,
};

// TODO(prod): Logout route should remove the device and pending notification
// tokens.
// TOOD(microservice): Most likely public ID will not be sent from account
// to other servers.

pub const PATH_POST_SET_DEVICE_TOKEN: &str = "/chat_api/set_device_token";

#[utoipa::path(
    post,
    path = PATH_POST_SET_DEVICE_TOKEN,
    request_body(content = FcmDeviceToken),
    responses(
        (status = 200, description = "Success.", body = PendingNotificationToken),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_set_device_token<S: WriteData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(device_token): Json<FcmDeviceToken>,
) -> Result<Json<PendingNotificationToken>, StatusCode> {
    CHAT.post_set_device_token.incr();

    let pending_notification_token = db_write!(state, move |cmds| {
        cmds.chat()
            .push_notifications()
            .set_device_token(id, device_token)
    })?;

    Ok(pending_notification_token.into())
}

pub const PATH_POST_GET_PENDING_NOTIFICATION: &str = "/chat_api/get_pending_notification";

/// Get pending notification and reset pending notification.
///
/// Requesting this route is always valid to avoid figuring out device
/// token values more easily.
#[utoipa::path(
    post,
    path = PATH_POST_GET_PENDING_NOTIFICATION,
    request_body(content = PendingNotificationToken),
    responses(
        (status = 200, description = "Success", body = PendingNotificationWithData),
    ),
    security(), // This is public route handler
)]
pub async fn post_get_pending_notification<S: GetAccounts + WriteData + ReadData>(
    State(state): State<S>,
    Json(token): Json<PendingNotificationToken>,
) -> Json<PendingNotificationWithData> {
    CHAT.post_get_pending_notification.incr();

    let result = db_write!(state, move |cmds| {
        cmds.chat()
            .push_notifications()
            .get_and_reset_pending_notification_with_notification_token(token)
    });

    let (id, notification_value) = match result {
        Ok((id, notification_value)) => (id, notification_value),
        Err(_) => return PendingNotificationWithData::default().into(),
    };

    let flags = PendingNotificationFlags::from(notification_value);
    let sender_info = if flags == PendingNotificationFlags::NEW_MESSAGE {
        state.read().chat().all_pending_message_sender_public_ids(id).await.ok()
    }   else {
        None
    };

    PendingNotificationWithData {
        value: notification_value,
        new_message_received_from: sender_info,
    }.into()
}

pub fn push_notification_router_private<S: StateBase + WriteData>(s: S) -> Router {
    use axum::routing::post;

    Router::new()
        .route(PATH_POST_SET_DEVICE_TOKEN, post(post_set_device_token::<S>))
        .with_state(s)
}

pub fn push_notification_router_public<S: StateBase + GetAccounts + WriteData + ReadData>(s: S) -> Router {
    use axum::routing::post;

    Router::new()
        .route(
            PATH_POST_GET_PENDING_NOTIFICATION,
            post(post_get_pending_notification::<S>),
        )
        .with_state(s)
}

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_PUSH_NOTIFICATION_COUNTERS_LIST,
    post_set_device_token,
    post_get_pending_notification,
);
