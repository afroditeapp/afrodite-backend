pub mod data;

use axum::{extract::Path, Json, TypedHeader};

use hyper::StatusCode;

use self::data::{Profile, Location};

use super::{model::AccountIdLight, utils::{}, GetUsers};

use tracing::error;

use super::{utils::ApiKeyHeader, GetApiKeys, ReadDatabase, WriteDatabase};

// TODO: Add timeout for database commands

pub const PATH_GET_PROFILE: &str = "/profile_api/profile/:account_id";

/// Get account's current profile.
///
/// Profile can include version UUID which can be used for caching.
///
/// # Access
/// Public profile access requires `view_public_profiles` capability.
/// Public and private profile access requires `admin_view_all_profiles`
/// capablility.
///
/// # Microservice notes
/// If account feature is set as external service then cached capability
/// information from account service is used for access checks.
#[utoipa::path(
    get,
    path = "/profile_api/profile/{account_id}",
    params(AccountIdLight),
    responses(
        (status = 200, description = "Get current profile.", body = Profile),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Profile does not exist, is private or other server error.",
        ),
    ),
    security(("api_key" = [])),
)]
pub async fn get_profile<S: ReadDatabase + GetUsers>(
    Path(requested_profile): Path<AccountIdLight>,
    state: S,
) -> Result<Json<Profile>, StatusCode> {
    // TODO: Validate user id

    // TODO: check capablities
    
    let requested_profile = state.users().get_internal_id(requested_profile).await.map_err(|e| {
        error!("get_profile: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    state
        .read_database()
        .read_json::<Profile>(requested_profile)
        .await
        .map(|profile| profile.into())
        .map_err(|e| {
            error!("get_profile: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
        })
}

/// TODO: Remove this after benchmarking?
pub const PATH_GET_DEFAULT_PROFILE: &str = "/profile_api/default/:account_id";


/// TODO: Remove this at some point
#[utoipa::path(
    get,
    path = "/profile_api/default/{account_id}",
    params(AccountIdLight),
    responses(
        (status = 200, description = "Get default profile.", body = Profile),
        (status = 401, description = "Unauthorized."),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_default_profile<S: ReadDatabase>(
    Path(_account_id): Path<AccountIdLight>,
    _state: S,
) -> Result<Json<Profile>, StatusCode> {
    let default = Profile::default();
    Ok(default.into())
}

pub const PATH_POST_PROFILE: &str = "/profile_api/profile";

/// Update profile information.
///
/// Writes the profile to the database only if it is changed.
///
/// TODO: string lenght validation, limit saving new profiles
#[utoipa::path(
    post,
    path = "/profile_api/profile",
    request_body = Profile,
    responses(
        (status = 200, description = "Update profile"),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Profile validation in route handler failed or database error."
        ),
    ),
    security(("api_key" = [])),
)]
pub async fn post_profile<S: GetApiKeys + WriteDatabase + ReadDatabase>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Json(mut profile): Json<Profile>,
    state: S,
) -> Result<(), StatusCode> {
    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let mut old_profile: Profile =
        state.read_database()
        .read_json(account_id)
        .await
        .map_err(|e| {
            error!("post_profile: read current profile, {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
        })?;

    old_profile.remove_version();
    profile.remove_version();

    if profile == old_profile {
        return Ok(())
    }

    profile.generate_new_version();

    state.write_database()
        .update_json(account_id, &profile)
        .await
        .map_err(|e| {
            error!("post_profile: write profile, {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
        })?;

    Ok(())
}


pub const PATH_PUT_LOCATION: &str = "/profile_api/location";

/// Update location
#[utoipa::path(
    put,
    path = "/profile_api/location",
    request_body = Location,
    responses(
        (status = 200, description = "Update successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn put_location<S: GetApiKeys + WriteDatabase + ReadDatabase>(
    Json(location): Json<Location>,
    state: S,
) -> Result<(), StatusCode> {

    Ok(())
}


pub const PATH_GET_NEXT_PROFILE_PAGE: &str = "/profile_api/page/next";

/// Get next page of profile list.
#[utoipa::path(
    get,
    path = "/profile_api/page/next",
    responses(
        (status = 200, description = "Update successfull.", body = ProfilePage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn get_next_profile_page<S: GetApiKeys + WriteDatabase + ReadDatabase>(
    Json(location): Json<Location>,
    state: S,
) -> Result<(), StatusCode> {

    Ok(())
}


pub const PATH_RESET_PROFILE_PAGING: &str = "/profile_api/page/reset";

/// Reset profile paging.
///
/// After this request getting next profiles will continue from the nearest
/// profiles.
#[utoipa::path(
    post,
    path = "/profile_api/page/reset",
    responses(
        (status = 200, description = "Update successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn post_reset_profile_paging<S: GetApiKeys + WriteDatabase + ReadDatabase>(
    state: S,
) -> Result<(), StatusCode> {

    Ok(())
}
