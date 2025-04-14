use axum::{extract::State, Extension};
use model::{
    AccountIdInternal, FcmDeviceToken, PendingNotificationToken, PendingNotificationWithData,
};
use crate::{create_open_api_router, db_write_multiple, S};
use simple_backend::create_counters;
use server_data::write::GetWriteCommandsCommon;

use super::super::utils::{Json, StatusCode};
use crate::app::WriteData;

// TODO(prod): Logout route should remove the device and pending notification
// tokens.
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

    let pending_notification_token = db_write_multiple!(state, move |cmds| {
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

    let result = db_write_multiple!(state, move |cmds| {
        cmds.common()
            .push_notification()
            .get_and_reset_pending_notification_with_notification_token(token)
            .await
    });

    let (id, notification_value) = match result {
        Ok((id, notification_value)) => (id, notification_value),
        Err(_) => return PendingNotificationWithData::default().into(),
    };

    state
        .data_all_access()
        .get_push_notification_data(id, notification_value)
        .await
        .into()
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
