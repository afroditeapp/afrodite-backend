use axum::{
    extract::State, Extension
};
use model_profile::{
    AccountIdInternal, ProfileAppNotificationSettings
};
use server_api::{
    app::WriteData, create_open_api_router, db_write_multiple, S
};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::ReadData;

const PATH_GET_PROFILE_APP_NOTIFICATION_SETTINGS: &str = "/profile_api/get_profile_app_notification_settings";

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

const PATH_POST_PROFILE_APP_NOTIFICATION_SETTINGS: &str = "/profile_api/post_profile_app_notification_settings";

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
        cmds.profile().notification().upsert_app_notification_settings(id, settings).await
    })?;
    Ok(())
}

create_open_api_router!(fn router_notification, get_profile_app_notification_settings, post_profile_app_notification_settings,);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_NOTIFICATION_COUNTERS_LIST,
    get_profile_app_notification_settings,
    post_profile_app_notification_settings,
);
