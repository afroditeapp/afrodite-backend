use axum::{Extension, extract::State};
use model_profile::{AccountIdInternal, ProfilePrivacySettings};
use server_api::{S, app::WriteData, create_open_api_router, db_write};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;

use super::super::utils::{Json, StatusCode};
use crate::app::ReadData;

const PATH_GET_PROFILE_PRIVACY_SETTINGS: &str = "/profile_api/get_profile_privacy_settings";

#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_PRIVACY_SETTINGS,
    responses(
        (status = 200, description = "Success.", body = ProfilePrivacySettings),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn get_profile_privacy_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<ProfilePrivacySettings>, StatusCode> {
    PROFILE.get_profile_privacy_settings.incr();

    let settings = state
        .read()
        .profile()
        .privacy()
        .profile_privacy_settings(id)
        .await?;

    Ok(settings.into())
}

const PATH_POST_PROFILE_PRIVACY_SETTINGS: &str = "/profile_api/post_profile_privacy_settings";

#[utoipa::path(
    post,
    path = PATH_POST_PROFILE_PRIVACY_SETTINGS,
    request_body = ProfilePrivacySettings,
    responses(
        (status = 200, description = "Success."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
async fn post_profile_privacy_settings(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
    Json(settings): Json<ProfilePrivacySettings>,
) -> Result<(), StatusCode> {
    PROFILE.post_profile_privacy_settings.incr();
    db_write!(state, move |cmds| {
        cmds.profile()
            .privacy()
            .upsert_privacy_settings(id, settings)
            .await
    })?;
    Ok(())
}

create_open_api_router!(
    fn router_privacy,
    get_profile_privacy_settings,
    post_profile_privacy_settings,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_PRIVACY_COUNTERS_LIST,
    get_profile_privacy_settings,
    post_profile_privacy_settings,
);
