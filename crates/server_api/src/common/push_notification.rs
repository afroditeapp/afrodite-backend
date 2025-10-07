use axum::{Extension, extract::State};
use base64::{Engine, prelude::BASE64_STANDARD};
use model::{
    AccountIdInternal, GetVapidPublicKey, PendingNotificationToken, PendingNotificationWithData,
    PushNotificationDeviceToken,
};
use server_data::{app::GetConfig, write::GetWriteCommandsCommon};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::{S, app::WriteData, create_open_api_router, db_write};

const PATH_POST_SET_DEVICE_TOKEN: &str = "/common_api/set_device_token";

#[utoipa::path(
    post,
    path = PATH_POST_SET_DEVICE_TOKEN,
    request_body(content = PushNotificationDeviceToken),
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
    Json(device_token): Json<PushNotificationDeviceToken>,
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

const PATH_GET_VAPID_PUBLIC_KEY: &str = "/common_api/get_vapid_public_key";

#[utoipa::path(
    get,
    path = PATH_GET_VAPID_PUBLIC_KEY,
    responses(
        (status = 200, description = "Success.", body = GetVapidPublicKey),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_vapid_public_key(
    State(state): State<S>,
) -> Result<Json<GetVapidPublicKey>, StatusCode> {
    COMMON.get_vapid_public_key.incr();

    let vapid_public_key =
        if let Some((_, vapid_builder)) = state.config().simple_backend().web_push_config() {
            Some(BASE64_STANDARD.encode(vapid_builder.get_public_key()))
        } else {
            None
        };

    let key = GetVapidPublicKey { vapid_public_key };

    Ok(key.into())
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
    State(_state): State<S>,
    Json(_token): Json<PendingNotificationToken>,
) -> Json<PendingNotificationWithData> {
    COMMON.post_get_pending_notification.incr();

    PendingNotificationWithData::default().into()
}

create_open_api_router!(fn router_push_notification_private, post_set_device_token, get_vapid_public_key,);

create_open_api_router!(fn router_push_notification_public, post_get_pending_notification,);

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_PUSH_NOTIFICATION_COUNTERS_LIST,
    post_set_device_token,
    get_vapid_public_key,
    post_get_pending_notification,
);
