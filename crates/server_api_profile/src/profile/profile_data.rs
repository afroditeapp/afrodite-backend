use axum::{
    extract::{Path, Query, State},
    Extension, Router,
};
use model::{
    AccountId, AccountIdInternal, AccountState, Capabilities, GetProfileQueryParam, GetProfileResult, ProfileSearchAgeRange, ProfileSearchAgeRangeValidated, ProfileUpdate, ProfileUpdateInternal, SearchGroups, ValidatedSearchGroups
};
use server_api::{app::IsMatch, db_write_multiple};
use server_data::read::GetReadCommandsCommon;
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;
use simple_backend_utils::IntoReportFromString;

use crate::{
    app::{
        GetAccessTokens, GetAccounts, GetConfig, GetInternalApi, ReadData, StateBase, WriteData,
    },
    db_write,
    utils::{Json, StatusCode},
    DataError,
};

// TODO: Add timeout for database commands

pub const PATH_GET_PROFILE: &str = "/profile_api/profile/:account_id";

/// Get account's current profile.
///
/// Response includes version UUID which can be used for caching.
///
/// # Access
///
/// ## Own profile
/// Unrestricted access.
///
/// ## Public other profiles
/// Normal account state required.
///
/// ## Private other profiles
/// If the profile is a match, then the profile can be accessed if query
/// parameter `is_match` is set to `true`.
///
/// If the profile is not a match, then capability `admin_view_all_profiles`
/// is required.
///
/// # Microservice notes
/// If account feature is set as external service then cached capability
/// information from account service is used for access checks.
#[utoipa::path(
    get,
    path = "/profile_api/profile/{account_id}",
    params(AccountId, GetProfileQueryParam),
    responses(
        (status = 200, description = "Get current profile", body = GetProfileResult),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile<
    S: ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData + GetConfig + IsMatch,
>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
    Extension(capabilities): Extension<Capabilities>,
    Path(requested_profile): Path<AccountId>,
    Query(params): Query<GetProfileQueryParam>,
) -> Result<Json<GetProfileResult>, StatusCode> {
    PROFILE.get_profile.incr();

    let requested_profile = state.get_internal_id(requested_profile).await?;

    let read_profile_action = || async {
        let profile_info = state
            .read()
            .profile()
            .profile(requested_profile)
            .await?;
        match params.profile_version() {
            Some(param_version) if param_version == profile_info.version =>
                Ok(GetProfileResult::current_version_latest_response(
                    profile_info.version,
                    profile_info.last_seen_time,
                ).into()),
            _ => Ok(GetProfileResult::profile_with_version_response(
                profile_info,
            ).into()),
        }
    };

    if account_id.as_id() == requested_profile.as_id() {
        return read_profile_action().await;
    }

    if account_state != AccountState::Normal {
        return Ok(GetProfileResult::empty().into());
    }

    let visibility = state
        .read()
        .common()
        .account(requested_profile)
        .await?
        .profile_visibility()
        .is_currently_public();

    if visibility ||
        capabilities.admin_view_all_profiles ||
        (params.allow_get_profile_if_match() && state.is_match(account_id, requested_profile).await?)
    {
        read_profile_action().await
    } else {
        Ok(GetProfileResult::empty().into())
    }
}

pub const PATH_POST_PROFILE: &str = "/profile_api/profile";

/// Update profile information.
///
/// Writes the profile to the database only if it is changed.
///
/// # Requirements
/// - Profile attributes must be valid
/// - Profile text must be empty
/// - Profile age must match with currently valid age range. The first min
///   value for the age range is the age at the initial setup. The second min
///   and max value is calculated using the following algorithm:
///  - The initial age (initialAge) is paired with the year of initial
///    setup completed (initialSetupYear).
///    - Year difference (yearDifference = currentYear - initialSetupYear) is
///      used for changing the range min and max.
///      - Min value: initialAge + yearDifference - 1.
///      - Max value: initialAge + yearDifference + 1.
///
/// TODO: string lenght validation, limit saving new profiles
/// TODO: return the new proifle. Edit: is this really needed?
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
pub async fn post_profile<S: GetConfig + GetAccessTokens + WriteData + ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(profile): Json<ProfileUpdate>,
) -> Result<(), StatusCode> {
    PROFILE.post_profile.incr();

    db_write_multiple!(state, move |cmds| {
        let account_state = cmds.read().common().account(account_id).await?.state();
        let old_profile = cmds.read().profile().profile(account_id).await?;
        let accepted_ages = if account_state != AccountState::InitialSetup {
            cmds.read().profile().accepted_profile_ages(account_id).await?
        } else {
            None
        };
        let profile = profile
            .validate(cmds.config().profile_attributes(), &old_profile.profile, accepted_ages)
            .into_error_string(DataError::NotAllowed)?;

        if profile.equals_with(&old_profile.profile) {
            return Ok(());
        }

        let new = ProfileUpdateInternal::new(profile);

        cmds.profile().profile(account_id, new).await?;

        Ok(())
    })?;

    Ok(())
}

pub const PATH_GET_SEARCH_GROUPS: &str = "/profile_api/search_groups";

/// Get account's current search groups
/// (gender and what gender user is looking for)
#[utoipa::path(
    get,
    path = "/profile_api/search_groups",
    responses(
        (status = 200, description = "Successful.", body = SearchGroups),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_search_groups<S: ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<SearchGroups>, StatusCode> {
    PROFILE.get_search_groups.incr();
    let profile_state = state.read().profile().profile_state(account_id).await?;
    Ok(Json(profile_state.search_group_flags.into()))
}

pub const PATH_POST_SEARCH_GROUPS: &str = "/profile_api/search_groups";

/// Set account's current search groups
/// (gender and what gender user is looking for)
#[utoipa::path(
    post,
    path = "/profile_api/search_groups",
    request_body = SearchGroups,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_search_groups<S: WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(search_groups): Json<SearchGroups>,
) -> Result<(), StatusCode> {
    PROFILE.post_search_groups.incr();

    let validated: ValidatedSearchGroups = search_groups
        .try_into()
        .into_error_string(DataError::NotAllowed)?;

    db_write!(state, move |cmds| cmds
        .profile()
        .update_search_groups(account_id, validated))
}

pub const PATH_GET_SEARCH_AGE_RANGE: &str = "/profile_api/search_age_range";

/// Get account's current search age range
#[utoipa::path(
    get,
    path = "/profile_api/search_age_range",
    responses(
        (status = 200, description = "Successful.", body = ProfileSearchAgeRange),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_search_age_range<S: ReadData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ProfileSearchAgeRange>, StatusCode> {
    PROFILE.get_search_age_range.incr();
    let profile_state = state.read().profile().profile_state(account_id).await?;
    Ok(Json(profile_state.into()))
}

pub const PATH_POST_SEARCH_AGE_RANGE: &str = "/profile_api/search_age_range";

/// Set account's current search age range
#[utoipa::path(
    post,
    path = "/profile_api/search_age_range",
    request_body = ProfileSearchAgeRange,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_search_age_range<S: WriteData>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(search_age_range): Json<ProfileSearchAgeRange>,
) -> Result<(), StatusCode> {
    PROFILE.post_search_age_range.incr();

    let validated: ProfileSearchAgeRangeValidated = search_age_range
        .try_into()
        .into_error_string(DataError::NotAllowed)?;

    db_write!(state, move |cmds| cmds
        .profile()
        .update_search_age_range(account_id, validated))
}

pub fn profile_data_router<
    S: StateBase + ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData + GetConfig + IsMatch,
>(
    s: S,
) -> Router {
    use axum::routing::{get, post};

    Router::new()
        .route(PATH_GET_PROFILE, get(get_profile::<S>))
        .route(PATH_GET_SEARCH_GROUPS, get(get_search_groups::<S>))
        .route(PATH_GET_SEARCH_AGE_RANGE, get(get_search_age_range::<S>))
        .route(PATH_POST_PROFILE, post(post_profile::<S>))
        .route(PATH_POST_SEARCH_GROUPS, post(post_search_groups::<S>))
        .route(PATH_POST_SEARCH_AGE_RANGE, post(post_search_age_range::<S>))
        .with_state(s)
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_DATA_COUNTERS_LIST,
    get_profile,
    get_search_groups,
    get_search_age_range,
    post_profile,
    post_search_groups,
    post_search_age_range,
);
