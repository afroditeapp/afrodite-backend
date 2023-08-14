use axum::{extract::Path, TypedHeader};
use hyper::StatusCode;
use tracing::error;

use model::{
    AccountIdLight, Location, Profile, ProfileInternal, ProfilePage, ProfileUpdate,
    ProfileUpdateInternal,
};

use super::{
    db_write, GetApiKeys, GetConfig, GetInternalApi, GetUsers, ReadDatabase, utils::{ApiKeyHeader, Json},
    WriteData,
};

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
pub async fn get_profile<S: ReadDatabase + GetUsers + GetApiKeys + GetInternalApi + WriteData + GetConfig>(
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
            .profile()
            .profile(requested_profile)
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
                .write(move |cmds| async move {
                    cmds.profile()
                        .profile_update_visibility(requested_profile, visibility, true)
                        .await
                })
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
            .profile()
            .profile(requested_profile)
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
pub async fn post_profile<S: GetApiKeys + WriteData + ReadDatabase>(
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
            .profile()
            .profile(account_id)
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

    db_write!(state, move |cmds| cmds.profile().profile(account_id, new))
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
pub async fn put_location<S: GetApiKeys + WriteData>(
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
        .write(move |cmds| async move {
            cmds.profile()
                .profile_update_location(account_id, location)
                .await
        })
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
pub async fn post_get_next_profile_page<S: GetApiKeys + WriteData>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    state: S,
) -> Result<Json<ProfilePage>, StatusCode> {
    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let data = state
        .write_concurrent(account_id.as_light(), move |cmds| async move {
            cmds.next_profiles(account_id).await
        })
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
pub async fn post_reset_profile_paging<S: GetApiKeys + WriteData + ReadDatabase>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    state: S,
) -> Result<(), StatusCode> {
    let account_id = state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;

    state
        .write(move |cmds| async move {
            cmds.profile()
                .profile_update_location(account_id, Location::default())
                .await
        })
        .await
        .map_err(|e| {
            error!("post_reset_profile_paging, {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    state
        .write_concurrent(account_id.as_light(), move |cmds| async move {
            cmds.reset_profile_iterator(account_id).await
        })
        .await
        .map_err(|e| {
            error!("post_reset_profile_paging, {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}

// ------------------- Benchmark routes ----------------------------

pub const PATH_GET_PROFILE_FROM_DATABASE_BENCHMARK: &str =
    "/profile_api/benchmark/profile/:account_id";

/// Get account's current profile from database. Debug mode must be enabled
/// that route can be used.
#[utoipa::path(
    get,
    path = "/profile_api/benchmark/profile/{account_id}",
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
pub async fn get_profile_from_database_debug_mode_benchmark<
    S: ReadDatabase + GetUsers + GetApiKeys + GetInternalApi + WriteData + GetConfig,
>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Path(requested_profile): Path<AccountIdLight>,
    state: S,
) -> Result<Json<Profile>, StatusCode> {
    if !state.config().debug_mode() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

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
            .profile()
            .read_profile_directly_from_database(requested_profile)
            .await
            .map(|profile| {
                let profile: Profile = profile.into();
                profile.into()
            })
            .map_err(|e| {
                error!("get_profile: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            });
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub const PATH_POST_PROFILE_TO_DATABASE_BENCHMARK: &str = "/profile_api/benchmark/profile";

/// Post account's current profile directly to database. Debug mode must be enabled
/// that route can be used.
#[utoipa::path(
    post,
    path = "/profile_api/benchmark/profile",
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
pub async fn post_profile_to_database_debug_mode_benchmark<
    S: GetApiKeys + WriteData + ReadDatabase,
>(
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
            .profile()
            .profile(account_id)
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

    db_write!(state, move |cmds| {
        cmds.profile()
            .benchmark_update_profile_bypassing_cache(account_id, new)
    })
    .await
    .map_err(|e| {
        error!("post_profile: write profile, {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
    })?;

    Ok(())
}
