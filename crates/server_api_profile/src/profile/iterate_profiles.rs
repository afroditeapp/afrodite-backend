use axum::{Extension, extract::State};
use model_profile::{AccountIdInternal, AutomaticProfileSearchSettings};
use server_api::{S, create_open_api_router, db_write};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;

use crate::{
    app::{ReadData, WriteData},
    utils::{Json, StatusCode},
};

const PATH_GET_AUTOMATIC_PROFILE_SEARCH_SETTINGS: &str =
    "/profile_api/automatic_profile_search_settings";

#[utoipa::path(
    get,
    path = PATH_GET_AUTOMATIC_PROFILE_SEARCH_SETTINGS,
    responses(
        (status = 200, description = "Successfull.", body = AutomaticProfileSearchSettings),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_automatic_profile_search_settings(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<AutomaticProfileSearchSettings>, StatusCode> {
    PROFILE.get_automatic_profile_search_settings.incr();
    let settings = state
        .read()
        .profile()
        .search()
        .automatic_profile_search_settings(account_id)
        .await?;
    Ok(settings.into())
}

const PATH_POST_AUTOMATIC_PROFILE_SEARCH_SETTINGS: &str =
    "/profile_api/automatic_profile_search_settings";

#[utoipa::path(
    post,
    path = PATH_POST_AUTOMATIC_PROFILE_SEARCH_SETTINGS,
    request_body = AutomaticProfileSearchSettings,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_automatic_profile_search_settings(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(settings): Json<AutomaticProfileSearchSettings>,
) -> Result<(), StatusCode> {
    PROFILE.post_automatic_profile_search_settings.incr();
    db_write!(state, move |cmds| {
        cmds.profile()
            .search()
            .upsert_automatic_profile_search_settings(account_id, settings)
            .await
    })?;
    Ok(())
}

create_open_api_router!(
    fn router_iterate_profiles,
    get_automatic_profile_search_settings,
    post_automatic_profile_search_settings,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ITERATE_PROFILES_COUNTERS_LIST,
    get_automatic_profile_search_settings,
    post_automatic_profile_search_settings,
);
