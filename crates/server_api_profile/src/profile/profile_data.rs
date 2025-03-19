use axum::{
    extract::{Path, Query, State},
    Extension,
};
use model_profile::{
    AccountId, AccountIdInternal, AccountState, GetInitialProfileAgeInfoResult, GetMyProfileResult,
    GetProfileQueryParam, GetProfileResult, Permissions, ProfileSearchAgeRange,
    ProfileSearchAgeRangeValidated, ProfileUpdate, SearchGroups,
    ValidatedSearchGroups,
};
use server_api::{app::GetConfig, create_open_api_router, db_write_multiple, result::WrappedContextExt, S};
use server_data::read::GetReadCommandsCommon;
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;
use simple_backend_utils::IntoReportFromString;

use crate::{
    app::{GetAccounts, ReadData, WriteData},
    utils::{Json, StatusCode},
    DataError,
};

// TODO: Add timeout for database commands

const PATH_GET_PROFILE: &str = "/profile_api/profile/{aid}";

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
/// If the profile is not a match, then permission `admin_view_all_profiles`
/// is required.
///
/// # Microservice notes
/// If account feature is set as external service then cached permission
/// information from account service is used for access checks.
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE,
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
pub async fn get_profile(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
    Extension(permissions): Extension<Permissions>,
    Path(requested_profile): Path<AccountId>,
    Query(params): Query<GetProfileQueryParam>,
) -> Result<Json<GetProfileResult>, StatusCode> {
    PROFILE.get_profile.incr();

    let requested_profile = state.get_internal_id(requested_profile).await?;

    let read_profile_action = || async {
        let profile_info = state.read().profile().profile(requested_profile).await?;
        match params.profile_version() {
            Some(param_version) if param_version == profile_info.version => {
                Ok(GetProfileResult::current_version_latest_response(
                    profile_info.version,
                    profile_info.last_seen_time,
                )
                .into())
            }
            _ => Ok(GetProfileResult::profile_with_version_response(profile_info).into()),
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

    if visibility
        || permissions.admin_view_all_profiles
        || (params.allow_get_profile_if_match()
            && state
                .data_all_access()
                .is_match(account_id, requested_profile)
                .await?)
    {
        read_profile_action().await
    } else {
        Ok(GetProfileResult::empty().into())
    }
}

const PATH_POST_PROFILE: &str = "/profile_api/profile";

// TODO(prod): Consider requiring that the first letter in profile name is
//             a capital letter.

/// Update profile information.
///
/// Writes the profile to the database only if it is changed.
///
/// WebSocket event about profile change will not be emitted. The event
/// is emitted only from server side profile updates.
///
/// # Requirements
/// - Profile attributes must be valid.
/// - Profile text must be 2000 bytes or less.
/// - Profile text must be trimmed.
/// - Profile name changes are only possible when initial setup is ongoing.
/// - Profile name must be trimmed and not empty.
/// - Profile name must be 100 bytes or less.
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
    path = PATH_POST_PROFILE,
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
pub async fn post_profile(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(profile): Json<ProfileUpdate>,
) -> Result<(), StatusCode> {
    PROFILE.post_profile.incr();

    db_write_multiple!(state, move |cmds| {
        let account_state = cmds.read().common().account(account_id).await?.state();
        let old_profile = cmds.read().profile().profile(account_id).await?;
        let accepted_ages = if account_state != AccountState::InitialSetup {
            cmds.read()
                .profile()
                .accepted_profile_ages(account_id)
                .await?
        } else {
            None
        };
        let profile = profile
            .validate(
                cmds.config().profile_attributes(),
                &old_profile.profile,
                accepted_ages,
            )
            .into_error_string(DataError::NotAllowed)?;

        if profile.equals_with(&old_profile.profile) {
            return Ok(());
        }

        if account_state != AccountState::InitialSetup && profile.name != old_profile.profile.name {
            return Err(DataError::NotAllowed.report());
        }

        if profile.name.trim().is_empty() || profile.name != profile.name.trim() {
            return Err(DataError::NotAllowed.report());
        }

        cmds.profile().profile(account_id, profile).await?;

        Ok(())
    })?;

    Ok(())
}

const PATH_GET_SEARCH_GROUPS: &str = "/profile_api/search_groups";

/// Get account's current search groups
/// (gender and what gender user is looking for)
#[utoipa::path(
    get,
    path = PATH_GET_SEARCH_GROUPS,
    responses(
        (status = 200, description = "Successful.", body = SearchGroups),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_search_groups(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<SearchGroups>, StatusCode> {
    PROFILE.get_search_groups.incr();
    let profile_state = state.read().profile().profile_state(account_id).await?;
    Ok(Json(profile_state.search_group_flags.into()))
}

const PATH_POST_SEARCH_GROUPS: &str = "/profile_api/search_groups";

/// Set account's current search groups
/// (gender and what gender user is looking for)
#[utoipa::path(
    post,
    path = PATH_POST_SEARCH_GROUPS,
    request_body = SearchGroups,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_search_groups(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(search_groups): Json<SearchGroups>,
) -> Result<(), StatusCode> {
    PROFILE.post_search_groups.incr();

    let validated: ValidatedSearchGroups = search_groups
        .try_into()
        .into_error_string(DataError::NotAllowed)?;

    db_write_multiple!(state, move |cmds| cmds
        .profile()
        .update_search_groups(account_id, validated).await)
}

const PATH_GET_SEARCH_AGE_RANGE: &str = "/profile_api/search_age_range";

/// Get account's current search age range
#[utoipa::path(
    get,
    path = PATH_GET_SEARCH_AGE_RANGE,
    responses(
        (status = 200, description = "Successful.", body = ProfileSearchAgeRange),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_search_age_range(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ProfileSearchAgeRange>, StatusCode> {
    PROFILE.get_search_age_range.incr();
    let profile_state = state.read().profile().profile_state(account_id).await?;
    Ok(Json(profile_state.into()))
}

const PATH_POST_SEARCH_AGE_RANGE: &str = "/profile_api/search_age_range";

/// Set account's current search age range
#[utoipa::path(
    post,
    path = PATH_POST_SEARCH_AGE_RANGE,
    request_body = ProfileSearchAgeRange,
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_search_age_range(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(search_age_range): Json<ProfileSearchAgeRange>,
) -> Result<(), StatusCode> {
    PROFILE.post_search_age_range.incr();

    let validated: ProfileSearchAgeRangeValidated = search_age_range
        .try_into()
        .into_error_string(DataError::NotAllowed)?;

    db_write_multiple!(state, move |cmds| cmds
        .profile()
        .update_search_age_range(account_id, validated).await)
}

const PATH_GET_MY_PROFILE: &str = "/profile_api/my_profile";

/// Get my profile
#[utoipa::path(
    get,
    path = PATH_GET_MY_PROFILE,
    responses(
        (status = 200, description = "Get my profile", body = GetMyProfileResult),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_my_profile(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<GetMyProfileResult>, StatusCode> {
    PROFILE.get_my_profile.incr();

    let r = state.read().profile().my_profile(account_id).await?;

    Ok(r.into())
}

const PATH_GET_INITIAL_PROFILE_AGE_INFO: &str = "/profile_api/initial_profile_age_info";

/// Get initial profile age information which can be used for calculating
/// current accepted profile ages.
#[utoipa::path(
    get,
    path = PATH_GET_INITIAL_PROFILE_AGE_INFO,
    responses(
        (status = 200, description = "Success", body = GetInitialProfileAgeInfoResult),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_initial_profile_age_info(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<GetInitialProfileAgeInfoResult>, StatusCode> {
    PROFILE.get_initial_profile_age_info.incr();

    let info = state
        .read()
        .profile()
        .accepted_profile_ages(account_id)
        .await?;

    let r = GetInitialProfileAgeInfoResult { info };

    Ok(r.into())
}

create_open_api_router!(
        fn router_profile_data,
        get_profile,
        get_search_groups,
        get_search_age_range,
        post_profile,
        post_search_groups,
        post_search_age_range,
        get_my_profile,
        get_initial_profile_age_info,
);

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
    get_my_profile,
    get_initial_profile_age_info,
);
