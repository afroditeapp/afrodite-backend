use axum::{Extension, extract::State};
use model::{
    MediaContentModerationCompletedNotification, MediaContentModerationCompletedNotificationViewed,
    PushNotificationFlags,
};
use model_media::{AccountIdInternal, MediaAppNotificationSettings};
use server_api::{
    S,
    app::{EventManagerProvider, WriteData},
    create_open_api_router, db_write,
};
use server_data_media::{read::GetReadMediaCommands, write::GetWriteCommandsMedia};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::ReadData;

const PATH_GET_MEDIA_APP_NOTIFICATION_SETTINGS: &str =
    "/media_api/get_media_app_notification_settings";

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

const PATH_POST_MEDIA_APP_NOTIFICATION_SETTINGS: &str =
    "/media_api/post_media_app_notification_settings";

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
    db_write!(state, move |cmds| {
        cmds.media()
            .notification()
            .upsert_app_notification_settings(id, settings)
            .await
    })?;
    Ok(())
}

const PATH_POST_GET_MEDIA_CONTENT_MODERATION_COMPLETED_NOTIFICATION: &str =
    "/media_api/media_content_moderation_completed_notification";

/// Get media content moderation completed notification.
///
#[utoipa::path(
    post,
    path = PATH_POST_GET_MEDIA_CONTENT_MODERATION_COMPLETED_NOTIFICATION,
    responses(
        (status = 200, description = "Successfull.", body = MediaContentModerationCompletedNotification),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_media_content_moderation_completed_notification(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<MediaContentModerationCompletedNotification>, StatusCode> {
    MEDIA
        .post_get_media_content_moderation_completed_notification
        .incr();

    let mut info = state
        .read()
        .media()
        .notification()
        .media_content_moderation_completed(account_id)
        .await?;

    let visibility = state
        .event_manager()
        .remove_pending_push_notification_flags_from_cache(
            account_id,
            PushNotificationFlags::MEDIA_CONTENT_MODERATION_COMPLETED,
        )
        .await;
    info.hidden = visibility.hidden;

    Ok(info.into())
}

const PATH_POST_MARK_MEDIA_CONTENT_MODERATION_COMPLETED_NOTIFICATION_VIEWED: &str =
    "/media_api/mark_media_content_moderation_completed_notification_viewed";

/// The viewed values must be updated to prevent WebSocket code from sending
/// unnecessary event about new notification.
#[utoipa::path(
    post,
    path = PATH_POST_MARK_MEDIA_CONTENT_MODERATION_COMPLETED_NOTIFICATION_VIEWED,
    request_body = MediaContentModerationCompletedNotificationViewed,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_mark_media_content_moderation_completed_notification_viewed(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(viewed): Json<MediaContentModerationCompletedNotificationViewed>,
) -> Result<(), StatusCode> {
    MEDIA
        .post_mark_media_content_moderation_completed_notification_viewed
        .incr();

    db_write!(state, move |cmds| {
        cmds.media()
            .notification()
            .update_notification_viewed_values(account_id, viewed)
            .await
    })?;

    Ok(())
}

create_open_api_router!(fn router_notification, get_media_app_notification_settings, post_media_app_notification_settings, post_get_media_content_moderation_completed_notification, post_mark_media_content_moderation_completed_notification_viewed,);

create_counters!(
    MediaCounters,
    MEDIA,
    MEDIA_NOTIFICATION_COUNTERS_LIST,
    get_media_app_notification_settings,
    post_media_app_notification_settings,
    post_get_media_content_moderation_completed_notification,
    post_mark_media_content_moderation_completed_notification_viewed,
);
