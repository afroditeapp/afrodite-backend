use axum::{Extension, extract::State};
use model::{
    AutomaticProfileSearchCompletedNotification, AutomaticProfileSearchCompletedNotificationViewed,
    PendingNotificationFlags, ProfileTextModerationCompletedNotification,
    ProfileTextModerationCompletedNotificationViewed,
};
use model_profile::{AccountIdInternal, ProfileAppNotificationSettings};
use server_api::{
    S,
    app::{EventManagerProvider, WriteData},
    create_open_api_router, db_write_multiple,
};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::ReadData;

const PATH_GET_PROFILE_APP_NOTIFICATION_SETTINGS: &str =
    "/profile_api/get_profile_app_notification_settings";

#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_APP_NOTIFICATION_SETTINGS,
    responses(
        (status = 200, description = "Success.", body = ProfileAppNotificationSettings),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn get_profile_app_notification_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<ProfileAppNotificationSettings>, StatusCode> {
    PROFILE.get_profile_app_notification_settings.incr();

    let settings = state
        .read()
        .profile()
        .notification()
        .chat_app_notification_settings(id)
        .await?;

    Ok(settings.into())
}

const PATH_POST_PROFILE_APP_NOTIFICATION_SETTINGS: &str =
    "/profile_api/post_profile_app_notification_settings";

#[utoipa::path(
    post,
    path = PATH_POST_PROFILE_APP_NOTIFICATION_SETTINGS,
    request_body = ProfileAppNotificationSettings,
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn post_profile_app_notification_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(settings): Json<ProfileAppNotificationSettings>,
) -> Result<(), StatusCode> {
    PROFILE.post_profile_app_notification_settings.incr();
    db_write_multiple!(state, move |cmds| {
        cmds.profile()
            .notification()
            .upsert_app_notification_settings(id, settings)
            .await
    })?;
    Ok(())
}

const PATH_POST_GET_PROFILE_TEXT_MODERATION_COMPLETED_NOTIFICATION: &str =
    "/profile_api/profile_text_moderation_completed_notification";

/// Get profile text moderation completed notification.
///
#[utoipa::path(
    post,
    path = PATH_POST_GET_PROFILE_TEXT_MODERATION_COMPLETED_NOTIFICATION,
    responses(
        (status = 200, description = "Successfull.", body = ProfileTextModerationCompletedNotification),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_profile_text_moderation_completed_notification(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ProfileTextModerationCompletedNotification>, StatusCode> {
    PROFILE
        .post_get_profile_text_moderation_completed_notification
        .incr();

    let info = state
        .read()
        .profile()
        .notification()
        .profile_text_moderation_completed(account_id)
        .await?;

    state
        .event_manager()
        .remove_specific_pending_notification_flags_from_cache(
            account_id,
            PendingNotificationFlags::PROFILE_TEXT_MODERATION_COMPLETED,
        )
        .await;

    Ok(info.into())
}

const PATH_POST_MARK_PROFILE_TEXT_MODERATION_COMPLETED_NOTIFICATION_VIEWED: &str =
    "/profile_api/mark_profile_text_moderation_completed_notification_viewed";

/// The viewed values must be updated to prevent WebSocket code from sending
/// unnecessary event about new notification.
#[utoipa::path(
    post,
    path = PATH_POST_MARK_PROFILE_TEXT_MODERATION_COMPLETED_NOTIFICATION_VIEWED,
    request_body = ProfileTextModerationCompletedNotificationViewed,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_mark_profile_text_moderation_completed_notification_viewed(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(viewed): Json<ProfileTextModerationCompletedNotificationViewed>,
) -> Result<(), StatusCode> {
    PROFILE
        .post_mark_profile_text_moderation_completed_notification_viewed
        .incr();

    db_write_multiple!(state, move |cmds| {
        cmds.profile()
            .notification()
            .update_notification_viewed_values(account_id, viewed)
            .await
    })?;

    Ok(())
}

const PATH_POST_GET_AUTOMATIC_PROFILE_SEARCH_COMPLETED_NOTIFICATION: &str =
    "/profile_api/automatic_profile_search_completed_notification";

#[utoipa::path(
    post,
    path = PATH_POST_GET_AUTOMATIC_PROFILE_SEARCH_COMPLETED_NOTIFICATION,
    responses(
        (status = 200, description = "Successfull.", body = AutomaticProfileSearchCompletedNotification),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_automatic_profile_search_completed_notification(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<AutomaticProfileSearchCompletedNotification>, StatusCode> {
    PROFILE
        .post_get_automatic_profile_search_completed_notification
        .incr();

    let info = state
        .read()
        .profile()
        .notification()
        .automatic_profile_search_completed(account_id)
        .await?;

    state
        .event_manager()
        .remove_specific_pending_notification_flags_from_cache(
            account_id,
            PendingNotificationFlags::AUTOMATIC_PROFILE_SEARCH_COMPLETED,
        )
        .await;

    Ok(info.into())
}

const PATH_POST_MARK_AUTOMATIC_PROFILE_SEARCH_COMPLETED_NOTIFICATION_VIEWED: &str =
    "/profile_api/mark_automatic_profile_search_completed_notification_viewed";

/// The viewed values must be updated to prevent WebSocket code from sending
/// unnecessary event about new notification.
#[utoipa::path(
    post,
    path = PATH_POST_MARK_AUTOMATIC_PROFILE_SEARCH_COMPLETED_NOTIFICATION_VIEWED,
    request_body = AutomaticProfileSearchCompletedNotificationViewed,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_mark_automatic_profile_search_completed_notification_viewed(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(viewed): Json<AutomaticProfileSearchCompletedNotificationViewed>,
) -> Result<(), StatusCode> {
    PROFILE
        .post_mark_automatic_profile_search_completed_notification_viewed
        .incr();

    db_write_multiple!(state, move |cmds| {
        cmds.profile()
            .notification()
            .update_automatic_profile_search_notification_viewed_values(account_id, viewed)
            .await
    })?;

    Ok(())
}

create_open_api_router!(
   fn router_notification,
   get_profile_app_notification_settings,
   post_profile_app_notification_settings,
   post_get_profile_text_moderation_completed_notification,
   post_mark_profile_text_moderation_completed_notification_viewed,
   post_get_automatic_profile_search_completed_notification,
   post_mark_automatic_profile_search_completed_notification_viewed,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_NOTIFICATION_COUNTERS_LIST,
    get_profile_app_notification_settings,
    post_profile_app_notification_settings,
    post_get_profile_text_moderation_completed_notification,
    post_mark_profile_text_moderation_completed_notification_viewed,
    post_get_automatic_profile_search_completed_notification,
    post_mark_automatic_profile_search_completed_notification_viewed,
);
