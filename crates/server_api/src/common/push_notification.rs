use axum::{Extension, extract::State};
use model::{
    AccountIdInternal, FcmDeviceToken, PendingNotificationFlags, PendingNotificationToken,
    PendingNotificationWithData,
};
use server_data::write::GetWriteCommandsCommon;
use server_state::app::AdminNotificationProvider;
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::{S, app::WriteData, create_open_api_router, db_write};

// TODO(prod): Make sure that cache is updated when pending notification flags
//             are updated.
// TOOD(microservice): Most likely public ID will not be sent from account
// to other servers.

const PATH_POST_SET_DEVICE_TOKEN: &str = "/common_api/set_device_token";

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
pub async fn post_set_device_token(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(device_token): Json<FcmDeviceToken>,
) -> Result<Json<PendingNotificationToken>, StatusCode> {
    COMMON.post_set_device_token.incr();

    let pending_notification_token = db_write!(state, move |cmds| {
        cmds.common()
            .push_notification()
            .set_device_token(id, device_token)
            .await
    })?;

    Ok(pending_notification_token.into())
}

const PATH_POST_GET_PENDING_NOTIFICATION: &str = "/common_api/get_pending_notification";

/// Get pending notification and reset pending notification.
///
/// When client receives a FCM data notification use this API route
/// to download the notification.
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
pub async fn post_get_pending_notification(
    State(state): State<S>,
    Json(token): Json<PendingNotificationToken>,
) -> Json<PendingNotificationWithData> {
    COMMON.post_get_pending_notification.incr();

    let (id, mut data) = state
        .data_all_access()
        .get_push_notification_data(token)
        .await;

    if let Some(id) = id {
        let flags = PendingNotificationFlags::from(data.value);
        data.admin_notification = if flags.contains(PendingNotificationFlags::ADMIN_NOTIFICATION) {
            state.admin_notification().get_notification_state(id).await
        } else {
            None
        };
    }

    data.into()
}

create_open_api_router!(fn router_push_notification_private, post_set_device_token,);

create_open_api_router!(fn router_push_notification_public, post_get_pending_notification,);

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_PUSH_NOTIFICATION_COUNTERS_LIST,
    post_set_device_token,
    post_get_pending_notification,
);
