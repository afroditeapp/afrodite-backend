use axum::{
    extract::State, Extension
};
use model::{MediaContentModerationCompletedResult, PendingNotificationFlags};
use model_media::{
    AccountIdInternal, MediaAppNotificationSettings
};
use server_api::{
    app::{EventManagerProvider, WriteData}, create_open_api_router, db_write_multiple, S
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


const PATH_POST_GET_MEDIA_CONTENT_MODERATION_COMPLETED_RESULT: &str = "/media_api/media_content_moderation_completed_result";

/// Get media content moderation completed result.
///
#[utoipa::path(
    post,
    path = PATH_POST_GET_MEDIA_CONTENT_MODERATION_COMPLETED_RESULT,
    responses(
        (status = 200, description = "Successfull.", body = MediaContentModerationCompletedResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_media_content_moderation_completed(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<MediaContentModerationCompletedResult>, StatusCode> {
    MEDIA.post_get_media_content_moderation_completed.incr();

    let info = state.read().media().notification().media_content_moderation_completed(account_id).await?;

    db_write_multiple!(state, move |cmds| {
        cmds.media().notification().reset_notifications(account_id).await
    })?;

    state
        .event_manager()
        .remove_specific_pending_notification_flags_from_cache(
            account_id,
            PendingNotificationFlags::MEDIA_CONTENT_MODERATION_COMPLETED,
        )
        .await;

    Ok(info.into())
}

create_open_api_router!(fn router_notification, get_media_app_notification_settings, post_media_app_notification_settings, post_get_media_content_moderation_completed,);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_NOTIFICATION_COUNTERS_LIST,
    get_media_app_notification_settings,
    post_media_app_notification_settings,
    post_get_media_content_moderation_completed,
);
