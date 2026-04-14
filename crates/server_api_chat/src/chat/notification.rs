use axum::{Extension, extract::State};
use model::PushNotificationFlags;
use model_chat::{
    AccountIdInternal, ChatAppNotificationSettings, ChatEmailNotificationSettings,
    PendingChatNotification, PendingChatNotificationToDelete,
};
use server_api::{
    S,
    app::{EventManagerProvider, WriteData},
    create_open_api_router, db_write,
};
use server_data_chat::{read::GetReadChatCommands, write::GetWriteCommandsChat};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::ReadData;

const PATH_GET_CHAT_APP_NOTIFICATION_SETTINGS: &str =
    "/chat_api/get_chat_app_notification_settings";

#[utoipa::path(
    get,
    path = PATH_GET_CHAT_APP_NOTIFICATION_SETTINGS,
    responses(
        (status = 200, description = "Success.", body = ChatAppNotificationSettings),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn get_chat_app_notification_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<ChatAppNotificationSettings>, StatusCode> {
    CHAT.get_chat_app_notification_settings.incr();

    let settings = state
        .read()
        .chat()
        .notification()
        .chat_app_notification_settings(id)
        .await?;

    Ok(settings.into())
}

const PATH_POST_CHAT_APP_NOTIFICATION_SETTINGS: &str =
    "/chat_api/post_chat_app_notification_settings";

#[utoipa::path(
    post,
    path = PATH_POST_CHAT_APP_NOTIFICATION_SETTINGS,
    request_body = ChatAppNotificationSettings,
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn post_chat_app_notification_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(settings): Json<ChatAppNotificationSettings>,
) -> Result<(), StatusCode> {
    CHAT.post_chat_app_notification_settings.incr();
    db_write!(state, move |cmds| {
        cmds.chat()
            .notification()
            .upsert_app_notification_settings(id, settings)
            .await
    })?;
    Ok(())
}

const PATH_GET_CHAT_EMAIL_NOTIFICATION_SETTINGS: &str =
    "/chat_api/get_chat_email_notification_settings";

#[utoipa::path(
    get,
    path = PATH_GET_CHAT_EMAIL_NOTIFICATION_SETTINGS,
    responses(
        (status = 200, description = "Success.", body = ChatEmailNotificationSettings),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn get_chat_email_notification_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<ChatEmailNotificationSettings>, StatusCode> {
    CHAT.get_chat_email_notification_settings.incr();

    let settings = state
        .read()
        .chat()
        .notification()
        .chat_email_notification_settings(id)
        .await?;

    Ok(settings.into())
}

const PATH_POST_CHAT_EMAIL_NOTIFICATION_SETTINGS: &str =
    "/chat_api/post_chat_email_notification_settings";

#[utoipa::path(
    post,
    path = PATH_POST_CHAT_EMAIL_NOTIFICATION_SETTINGS,
    request_body = ChatEmailNotificationSettings,
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn post_chat_email_notification_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(settings): Json<ChatEmailNotificationSettings>,
) -> Result<(), StatusCode> {
    CHAT.post_chat_email_notification_settings.incr();
    db_write!(state, move |cmds| {
        cmds.chat()
            .notification()
            .upsert_email_notification_settings(id, settings)
            .await
    })?;
    Ok(())
}

const PATH_GET_PENDING_CHAT_NOTIFICATIONS: &str = "/chat_api/pending_notifications";

#[utoipa::path(
    get,
    path = PATH_GET_PENDING_CHAT_NOTIFICATIONS,
    responses(
        (status = 200, description = "Success.", body = Vec<PendingChatNotification>),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn get_pending_chat_notifications(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<Vec<PendingChatNotification>>, StatusCode> {
    CHAT.get_pending_chat_notifications.incr();

    let notifications = state
        .read()
        .chat()
        .notification()
        .pending_chat_notifications(id)
        .await?;

    state
        .event_manager()
        .remove_pending_push_notification_flags_from_cache(
            id,
            PushNotificationFlags::PENDING_CHAT_NOTIFICATION,
        )
        .await;

    Ok(notifications.into())
}

const PATH_POST_DELETE_PENDING_CHAT_NOTIFICATIONS: &str = "/chat_api/pending_notifications/delete";

#[utoipa::path(
    post,
    path = PATH_POST_DELETE_PENDING_CHAT_NOTIFICATIONS,
    request_body = Vec<PendingChatNotificationToDelete>,
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn post_delete_pending_chat_notifications(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(notifications): Json<Vec<PendingChatNotificationToDelete>>,
) -> Result<(), StatusCode> {
    CHAT.post_delete_pending_chat_notifications.incr();

    db_write!(state, move |cmds| {
        cmds.chat()
            .notification()
            .delete_pending_chat_notifications(id, notifications)
            .await
    })?;

    Ok(())
}

create_open_api_router!(
    fn router_notification,
    get_chat_app_notification_settings,
    post_chat_app_notification_settings,
    get_chat_email_notification_settings,
    post_chat_email_notification_settings,
    get_pending_chat_notifications,
    post_delete_pending_chat_notifications,
);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_NOTIFICATION_COUNTERS_LIST,
    get_chat_app_notification_settings,
    post_chat_app_notification_settings,
    get_chat_email_notification_settings,
    post_chat_email_notification_settings,
    get_pending_chat_notifications,
    post_delete_pending_chat_notifications,
);
