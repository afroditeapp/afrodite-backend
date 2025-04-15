use axum::{
    extract::State, Extension
};
use model_media::{
    AccountIdInternal, MediaAppNotificationSettings
};
use server_api::{
    app::WriteData, create_open_api_router, db_write_multiple, S
};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::ReadData;

const PATH_GET_MEDIA_APP_NOTIFICATION_SETTINGS: &str = "/media_api/get_media_app_notification_settings";

#[utoipa::path(
    get,
    path = PATH_GET_MEDIA_APP_NOTIFICATION_SETTINGS,
    responses(
        (status = 200, description = "Success.", body = MediaAppNotificationSettings),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn get_media_app_notification_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<MediaAppNotificationSettings>, StatusCode> {
    MEDIA.get_media_app_notification_settings.incr();

    let settings = state
        .read()
        .media()
        .notification()
        .chat_app_notification_settings(id)
        .await?;

    Ok(settings.into())
}

const PATH_POST_MEDIA_APP_NOTIFICATION_SETTINGS: &str = "/media_api/post_media_app_notification_settings";

#[utoipa::path(
    post,
    path = PATH_POST_MEDIA_APP_NOTIFICATION_SETTINGS,
    request_body = MediaAppNotificationSettings,
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn post_media_app_notification_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(settings): Json<MediaAppNotificationSettings>,
) -> Result<(), StatusCode> {
    MEDIA.post_media_app_notification_settings.incr();
    db_write_multiple!(state, move |cmds| {
        cmds.media().notification().upsert_app_notification_settings(id, settings).await
    })?;
    Ok(())
}

create_open_api_router!(fn router_notification, get_media_app_notification_settings, post_media_app_notification_settings,);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_NOTIFICATION_COUNTERS_LIST,
    get_media_app_notification_settings,
    post_media_app_notification_settings,
);
