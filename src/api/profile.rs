pub mod data;
pub mod internal;

use axum::{extract::Path, Json, TypedHeader};

use hyper::StatusCode;

use self::data::{
    Location, Profile, ProfileInternal, ProfilePage, ProfileUpdate, ProfileUpdateInternal,
};

use super::{model::AccountIdLight, GetInternalApi, GetUsers};

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
pub async fn get_profile<
    S: ReadDatabase + GetUsers + GetApiKeys + GetInternalApi + WriteDatabase,
>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Path(requested_profile): Path<AccountIdLight>,
    state: S,
) -> Result<Json<Profile>, StatusCode> {
    // TODO: check capablities

    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let requested_profile = state
        .users()
        .get_internal_id(requested_profile)
        .await
        .map_err(|e| {
            error!("get_profile: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if account_id.as_light() == requested_profile.as_light() {
        return state
            .read_database()
            .read_json::<ProfileInternal>(requested_profile)
            .await
            .map(|profile| {
                let profile: Profile = profile.into();
                profile.into()
            })
            .map_err(|e| {
                error!("get_profile: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            });
    }

    let visibility = state
        .read_database()
        .profile_visibility(requested_profile)
        .await
        .map_err(|e| {
            error!("get_profile: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let visiblity = match visibility {
        Some(v) => v,
        None => {
            let account = state
                .internal_api()
                .get_account_state(requested_profile)
                .await
                .map_err(|e| {
                    error!("get_profile: {e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            let visibility = account.capablities().view_public_profiles;
            state
                .write_database()
                .profile()
                .update_profile_visiblity(requested_profile, visibility, true)
                .await
                .map_err(|e| {
                    error!("get_profile: {e:?}");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            visibility
        }
    };

    if visiblity {
        state
            .read_database()
            .read_json::<ProfileInternal>(requested_profile)
            .await
            .map(|profile| {
                let profile: Profile = profile.into();
                profile.into()
            })
            .map_err(|e| {
                error!("get_profile: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
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
    request_body = ProfileUpdate,
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
    Json(profile): Json<ProfileUpdate>,
    state: S,
) -> Result<(), StatusCode> {
    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let old_profile: ProfileInternal =
        state
            .read_database()
            .read_json(account_id)
            .await
            .map_err(|e| {
                error!("post_profile: read current profile, {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
            })?;
    let old_profile: Profile = old_profile.into();

    if profile == old_profile.into_update() {
        return Ok(());
    }

    let new = ProfileUpdateInternal::new(profile);

    state
        .write_database()
        .profile()
        .update_profile(account_id, new)
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
pub async fn put_location<S: GetApiKeys + WriteDatabase>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Json(location): Json<Location>,
    state: S,
) -> Result<(), StatusCode> {
    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    state
        .write_database()
        .profile()
        .update_profile_location(account_id, location)
        .await
        .map_err(|e| {
            error!("put_location, {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}

pub const PATH_POST_NEXT_PROFILE_PAGE: &str = "/profile_api/page/next";

/// Post (updates iterator) to get next page of profile list.
#[utoipa::path(
    post,
    path = "/profile_api/page/next",
    responses(
        (status = 200, description = "Update successfull.", body = ProfilePage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn post_get_next_profile_page<S: GetApiKeys + WriteDatabase>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    state: S,
) -> Result<Json<ProfilePage>, StatusCode> {
    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let data = state
        .write_database()
        .next_profiles(account_id)
        .await
        .map_err(|e| {
            error!("put_location, {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(ProfilePage { profiles: data }.into())
}

pub const PATH_POST_RESET_PROFILE_PAGING: &str = "/profile_api/page/reset";

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
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    state: S,
) -> Result<(), StatusCode> {
    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    state
        .write_database()
        .reset_profile_iterator(account_id)
        .await
        .map_err(|e| {
            error!("post_reset_profile_paging, {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}
