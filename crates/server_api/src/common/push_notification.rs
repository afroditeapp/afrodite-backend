use axum::{Extension, extract::State};
use model::{
    AccountIdInternal, ClientType, GetPushNotificationInfo, PendingNotificationToken,
    PendingNotificationWithData, PushNotificationDeviceToken, VapidPublicKey,
};
use server_data::{
    app::{GetConfig, ReadData},
    read::GetReadCommandsCommon,
    write::GetWriteCommandsCommon,
};
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

const PATH_GET_PUSH_NOTIFICATION_INFO: &str = "/common_api/get_push_notification_info";

#[utoipa::path(
    get,
    path = PATH_GET_PUSH_NOTIFICATION_INFO,
    responses(
        (status = 200, description = "Success.", body = GetPushNotificationInfo),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_push_notification_info(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<GetPushNotificationInfo>, StatusCode> {
    COMMON.get_push_notification_info.incr();

    let db_state = state
        .read()
        .common()
        .push_notification()
        .push_notification_db_state(id)
        .await?;

    let client = state
        .read()
        .common()
        .client_config()
        .client_login_session_platform(id)
        .await?;
    let vapid_public_key = if let Some(ClientType::Web) = client
        && let Some((_, vapid_builder)) = state.config().simple_backend().web_push_config()
    {
        Some(VapidPublicKey::new(&vapid_builder.get_public_key()))
    } else {
        None
    };

    let sync_version = state
        .read()
        .common()
        .push_notification()
        .push_notification_info_sync_version(id)
        .await?;

    let key = GetPushNotificationInfo {
        device_token: db_state.push_notification_device_token,
        vapid_public_key,
        sync_version,
    };

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

create_open_api_router!(fn router_push_notification_private, post_set_device_token, get_push_notification_info,);

create_open_api_router!(fn router_push_notification_public, post_get_pending_notification,);

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_PUSH_NOTIFICATION_COUNTERS_LIST,
    post_set_device_token,
    get_push_notification_info,
    post_get_pending_notification,
);
