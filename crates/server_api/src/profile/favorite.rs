use axum::{extract::State, Extension, Router};
use model::{AccountId, AccountIdInternal, FavoriteProfilesPage};
use simple_backend::create_counters;

use crate::{
    app::{GetAccounts, ReadData, StateBase, WriteData}, db_write, utils::{Json, StatusCode}
};

pub const PATH_GET_FAVORITE_PROFILES: &str = "/profile_api/favorite_profiles";

/// Get list of all favorite profiles.
#[utoipa::path(
    get,
    path = "/profile_api/favorite_profiles",
    responses(
        (status = 200, description = "Get successfull.", body = FavoriteProfilesPage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_favorite_profiles<S: ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<FavoriteProfilesPage>, StatusCode> {
    PROFILE.get_favorite_profiles.incr();
    let profiles = state.read().profile().favorite_profiles(account_id).await?;

    let page = FavoriteProfilesPage {
        profiles: profiles.into_iter().map(|p| p.uuid).collect(),
    };

    Ok(page.into())
}

pub const PATH_POST_FAVORITE_PROFILE: &str = "/profile_api/favorite_profile";

/// Add new favorite profile
#[utoipa::path(
    post,
    path = "/profile_api/favorite_profile",
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Request successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_favorite_profile<S: WriteData + GetAccounts>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(favorite): Json<AccountId>,
) -> Result<(), StatusCode> {
    PROFILE.post_favorite_profile.incr();

    let favorite_account_id = state.get_internal_id(favorite).await?;
    db_write!(state, move |cmds| cmds
        .profile()
        .insert_favorite_profile(account_id, favorite_account_id))?;

    Ok(())
}

pub const PATH_DELETE_FAVORITE_PROFILE: &str = "/profile_api/favorite_profile";

/// Delete favorite profile
#[utoipa::path(
    delete,
    path = "/profile_api/favorite_profile",
    request_body(content = AccountId),
    responses(
        (status = 200, description = "Request successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn delete_favorite_profile<S: WriteData + GetAccounts>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(favorite): Json<AccountId>,
) -> Result<(), StatusCode> {
    PROFILE.delete_favorite_profile.incr();
    let favorite_account_id = state.get_internal_id(favorite).await?;
    db_write!(state, move |cmds| cmds
        .profile()
        .remove_favorite_profile(account_id, favorite_account_id))?;

    Ok(())
}

pub fn favorite_router<S: StateBase + WriteData + GetAccounts + ReadData>(s: S) -> Router {
    use axum::routing::{delete, get, post};

    Router::new()
        .route(PATH_GET_FAVORITE_PROFILES, get(get_favorite_profiles::<S>))
        .route(PATH_POST_FAVORITE_PROFILE, post(post_favorite_profile::<S>))
        .route(
            PATH_DELETE_FAVORITE_PROFILE,
            delete(delete_favorite_profile::<S>),
        )
        .with_state(s)
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_FAVORITE_COUNTERS_LIST,
    get_favorite_profiles,
    post_favorite_profile,
    delete_favorite_profile,
);
