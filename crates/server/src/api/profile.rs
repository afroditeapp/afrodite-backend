use axum::{extract::Path, Extension};
use model::{
    AccountId, AccountIdInternal, Location, Profile, ProfilePage, ProfileUpdate,
    ProfileUpdateInternal, ProfileLink,
};

use crate::data::{write_concurrent::{ConcurrentWriteProfileHandle, ConcurrentWriteAction}, DataError};

use super::{
    db_write,
    utils::{Json, StatusCode},
    GetAccessTokens, GetAccounts, GetConfig, GetInternalApi, ReadData, WriteData,
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
    params(AccountId),
    responses(
        (status = 200, description = "Get current profile.", body = Profile),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Profile does not exist, is private or other server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile<
    S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData + GetConfig,
>(
    Extension(account_id): Extension<AccountIdInternal>,
    Path(requested_profile): Path<AccountId>,
    state: S,
) -> Result<Json<Profile>, StatusCode> {
    // TODO: check capablities

    let requested_profile = state.accounts().get_internal_id(requested_profile).await?;

    if account_id.as_id() == requested_profile.as_id() {
        return state
            .read()
            .profile()
            .profile(requested_profile)
            .await
            .map_err(Into::into)
            .map(|p| Into::<Profile>::into(p).into());
    }

    let visibility = state.read().profile_visibility(requested_profile).await?;

    let visiblity = match visibility {
        Some(v) => v,
        None => {
            let account = state
                .internal_api()
                .get_account_state(requested_profile)
                .await?;
            let visibility = account.capablities().view_public_profiles;
            db_write!(state, move |cmds| {
                cmds.profile()
                    .profile_update_visibility(requested_profile, visibility, true)
            })?;

            visibility
        }
    };

    if visiblity {
        state
            .read()
            .profile()
            .profile(requested_profile)
            .await
            .map_err(Into::into)
            .map(|p| Into::<Profile>::into(p).into())
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
    security(("access_token" = [])),
)]
pub async fn post_profile<S: GetAccessTokens + WriteData + ReadData>(
    Extension(account_id): Extension<AccountIdInternal>,
    Json(profile): Json<ProfileUpdate>,
    state: S,
) -> Result<(), StatusCode> {
    let old_profile: Profile = state.read().profile().profile(account_id).await?.into();

    if profile == old_profile.into_update() {
        return Ok(());
    }

    let new = ProfileUpdateInternal::new(profile);

    db_write!(state, move |cmds| cmds.profile().profile(account_id, new))
}

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
    Extension(account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<Location>, StatusCode> {
    let location = state
        .read()
        .profile()
        .profile_location(account_id).await?;
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
    Extension(account_id): Extension<AccountIdInternal>,
    Json(location): Json<Location>,
    state: S,
) -> Result<(), StatusCode> {
    db_write!(state, move |cmds| cmds
        .profile()
        .profile_update_location(account_id, location))
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
    security(("access_token" = [])),
)]
pub async fn post_get_next_profile_page<S: GetAccessTokens + WriteData>(
    Extension(account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<Json<ProfilePage>, StatusCode> {
    let data = state
        .write_concurrent(account_id.as_id(), move |cmds| async move {
            let out: ConcurrentWriteAction<error_stack::Result<Vec<ProfileLink>, DataError>> = cmds.accquire_profile(
                move |cmds: ConcurrentWriteProfileHandle| Box::new(async move {
                    cmds.next_profiles(account_id).await
                })
            ).await;
            out
        })
        .await??;

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
    security(("access_token" = [])),
)]
pub async fn post_reset_profile_paging<S: GetAccessTokens + WriteData + ReadData>(
    Extension(account_id): Extension<AccountIdInternal>,
    state: S,
) -> Result<(), StatusCode> {
    db_write!(state, move |cmds| cmds
        .profile()
        .profile_update_location(account_id, Location::default()))?;

    state
        .write_concurrent(account_id.as_id(), move |cmds| async move {
            let out: ConcurrentWriteAction<error_stack::Result<_, DataError>> = cmds.accquire_profile(
                move |cmds: ConcurrentWriteProfileHandle| Box::new(async move {
                    cmds.reset_profile_iterator(account_id).await
                })
            ).await;
            out
        })
        .await??;

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
    params(AccountId),
    responses(
        (status = 200, description = "Get current profile.", body = Profile),
        (status = 401, description = "Unauthorized."),
        (
            status = 500,
            description = "Profile does not exist, is private or other server error.",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_from_database_debug_mode_benchmark<
    S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData + GetConfig,
>(
    Extension(account_id): Extension<AccountIdInternal>,
    Path(requested_profile): Path<AccountId>,
    state: S,
) -> Result<Json<Profile>, StatusCode> {
    if !state.config().debug_mode() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let requested_profile = state.accounts().get_internal_id(requested_profile).await?;

    if account_id.as_id() == requested_profile.as_id() {
        let profile: Profile = state
            .read()
            .profile()
            .read_profile_directly_from_database(requested_profile)
            .await?
            .into();
        Ok(profile.into())
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
    security(("access_token" = [])),
)]
pub async fn post_profile_to_database_debug_mode_benchmark<
    S: GetAccessTokens + WriteData + ReadData,
>(
    Extension(account_id): Extension<AccountIdInternal>,
    Json(profile): Json<ProfileUpdate>,
    state: S,
) -> Result<(), StatusCode> {
    let old_profile: Profile = state.read().profile().profile(account_id).await?.into();

    if profile == old_profile.into_update() {
        return Ok(());
    }

    let new = ProfileUpdateInternal::new(profile);

    db_write!(state, move |cmds| {
        cmds.profile()
            .benchmark_update_profile_bypassing_cache(account_id, new)
    })
}
