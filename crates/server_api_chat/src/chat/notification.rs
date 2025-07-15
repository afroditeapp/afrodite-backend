use axum::{Extension, extract::State};
use model_chat::{AccountIdInternal, ChatAppNotificationSettings};
use server_api::{S, app::WriteData, create_open_api_router, db_write};
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

create_open_api_router!(fn router_notification, get_chat_app_notification_settings, post_chat_app_notification_settings,);

create_counters!(
    ChatCounters,
    CHAT,
    CHAT_NOTIFICATION_COUNTERS_LIST,
    get_chat_app_notification_settings,
    post_chat_app_notification_settings,
);
