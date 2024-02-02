use axum::{extract::{Path, State}, Extension, Router};
use model::{
    AccountId, AccountIdInternal, FavoriteProfilesPage, Location, Profile, ProfileLink,
    ProfilePage, ProfileUpdate, ProfileUpdateInternal,
};
use simple_backend::create_counters;

use crate::app::{GetAccessTokens, GetAccounts, GetConfig, GetInternalApi, ReadData, WriteData};
use crate::api::{
    db_write,
    utils::{Json, StatusCode},
};
use crate::{data::{
    write_concurrent::{ConcurrentWriteAction, ConcurrentWriteProfileHandle},
    DataError,
}};


pub const PATH_GET_LOCATION: &str = "/profile_api/location";

/// Get location for account which makes this request.
#[utoipa::path(
    get,
    path = "/profile_api/location",
    responses(
        (status = 200, description = "Get successfull.", body = Location),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_location<S: GetAccessTokens + ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<Location>, StatusCode> {
    PROFILE.get_location.incr();

    let location = state.read().profile().profile_location(account_id).await?;
    Ok(location.into())
}

pub const PATH_PUT_LOCATION: &str = "/profile_api/location";

/// Update location for account which makes this request.
#[utoipa::path(
    put,
    path = "/profile_api/location",
    request_body = Location,
    responses(
        (status = 200, description = "Update successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn put_location<S: GetAccessTokens + WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(location): Json<Location>,
) -> Result<(), StatusCode> {
    PROFILE.put_location.incr();

    db_write!(state, move |cmds| cmds
        .profile()
        .profile_update_location(account_id, location))
}

pub fn location_router(s: crate::app::S) -> Router {
    use crate::app::S;
    use axum::routing::{get, post, delete, put};

    Router::new()
        .route(PATH_GET_LOCATION, get(get_location::<S>))
        .route(PATH_PUT_LOCATION, put(put_location::<S>))
        .with_state(s)
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_LOCATION_COUNTERS_LIST,
    get_location,
    put_location,
);
