use axum::{
    extract::{Path, State},
    Extension, Router,
};
use model::{AccountId, AccountIdInternal, Profile, ProfileUpdate, ProfileUpdateInternal};
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
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Path(requested_profile): Path<AccountId>,
) -> Result<Json<Profile>, StatusCode> {
    PROFILE
        .get_profile_from_database_debug_mode_benchmark
        .incr();

    if !state.config().debug_mode() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let requested_profile = state.get_internal_id(requested_profile).await?;

    if account_id.as_id() == requested_profile.as_id() {
        let profile: Profile = state
            .read()
            .profile()
            .benchmark_read_profile_directly_from_database(requested_profile)
            .await?;
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
    S: GetConfig + GetAccessTokens + WriteData + ReadData,
>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(profile): Json<ProfileUpdate>,
) -> Result<(), StatusCode> {
    PROFILE.post_profile_to_database_debug_mode_benchmark.incr();

    let profile = profile
        .validate(state.config().profile_attributes())
        .into_error_string(DataError::NotAllowed)?;
    let old_profile = state.read().profile().profile(account_id).await?;

    if profile.equals_with(&old_profile.profile) {
        return Ok(());
    }

    let new = ProfileUpdateInternal::new(profile);

    db_write!(state, move |cmds| {
        cmds.profile()
            .benchmark_update_profile_bypassing_cache(account_id, new)
    })
}

// ------------------- Benchmark routes end ----------------------------

pub fn benchmark_router<
    S: StateBase + ReadData + GetAccounts + GetAccessTokens + GetInternalApi + WriteData + GetConfig,
>(
    s: S,
) -> Router {
    use axum::routing::{get, post};

    Router::new()
        .route(
            PATH_GET_PROFILE_FROM_DATABASE_BENCHMARK,
            get(get_profile_from_database_debug_mode_benchmark::<S>),
        )
        .route(
            PATH_POST_PROFILE_TO_DATABASE_BENCHMARK,
            post(post_profile_to_database_debug_mode_benchmark::<S>),
        )
        .with_state(s)
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_BENCHMARK_COUNTERS_LIST,
    get_profile_from_database_debug_mode_benchmark,
    post_profile_to_database_debug_mode_benchmark,
);
