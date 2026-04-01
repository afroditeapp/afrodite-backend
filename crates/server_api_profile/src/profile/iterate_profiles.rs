use axum::{Extension, extract::State};
use model_profile::{
    AccountIdInternal, AutomaticProfileSearchIteratorSessionId, AutomaticProfileSearchSettings,
    ProfilePage,
};
use server_api::{
    S,
    app::{ApiLimitsProvider, ApiUsageTrackerProvider},
    create_open_api_router, db_write,
};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;

use crate::{
    app::{ReadData, WriteData},
    utils::{Json, StatusCode},
};

const PATH_POST_AUTOMATIC_PROFILE_SEARCH_GET_NEXT_PROFILE_PAGE: &str =
    "/profile_api/automatic_profile_search/next";

/// Post (updates iterator) to get next page of automatic profile search profile list.
#[utoipa::path(
    post,
    path = PATH_POST_AUTOMATIC_PROFILE_SEARCH_GET_NEXT_PROFILE_PAGE,
    request_body(content = AutomaticProfileSearchIteratorSessionId),
    responses(
        (status = 200, description = "Update successfull.", body = ProfilePage),
        (status = 401, description = "Unauthorized."),
        (status = 429, description = "Too many requests."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_automatic_profile_search_get_next_profile_page(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(iterator_session_id): Json<AutomaticProfileSearchIteratorSessionId>,
) -> Result<Json<ProfilePage>, StatusCode> {
    PROFILE
        .post_automatic_profile_search_get_next_profile_page
        .incr();
    state
        .api_usage_tracker()
        .incr(account_id, |u| {
            &u.post_automatic_profile_search_get_next_profile_page
        })
        .await;
    state
        .api_limits(account_id)
        .profile()
        .post_get_next_profile_page()
        .await?;

    if !state
        .read()
        .profile()
        .search()
        .automatic_profile_search_happened_at_least_once(account_id)
        .await?
    {
        // Automatic search not done yet
        return Ok(ProfilePage::successful(vec![]).into());
    }

    let data = state
        .concurrent_write_profile_blocking(account_id.as_id(), move |cmds| {
            cmds.automatic_profile_search_next_profiles(account_id, iterator_session_id)
        })
        .await??;

    if let Some(data) = data {
        // Profile iterator session ID was valid
        Ok(ProfilePage::successful(data).into())
    } else {
        Ok(ProfilePage::error_invalid_iterator_session_id().into())
    }
}

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
    post_automatic_profile_search_get_next_profile_page,
    get_automatic_profile_search_settings,
    post_automatic_profile_search_settings,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ITERATE_PROFILES_COUNTERS_LIST,
    post_automatic_profile_search_get_next_profile_page,
    get_automatic_profile_search_settings,
    post_automatic_profile_search_settings,
);
