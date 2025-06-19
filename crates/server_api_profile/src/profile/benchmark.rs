use axum::{
    Extension,
    extract::{Path, State},
};
use model_profile::{AccountId, AccountIdInternal, AccountState, Profile, ProfileUpdate};
use server_api::{S, create_open_api_router, db_write_multiple};
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
use simple_backend::create_counters;
use simple_backend_utils::IntoReportFromString;

use crate::{
    DataError,
    app::{GetAccounts, GetConfig, ReadData, WriteData},
    utils::{Json, StatusCode},
};

// ------------------- Benchmark routes ----------------------------

const PATH_GET_PROFILE_FROM_DATABASE_BENCHMARK: &str = "/profile_api/benchmark/profile/{aid}";

/// Get account's current profile from database. Debug mode must be enabled
/// that route can be used.
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_FROM_DATABASE_BENCHMARK,
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
pub async fn get_profile_from_database_debug_mode_benchmark(
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

const PATH_POST_PROFILE_TO_DATABASE_BENCHMARK: &str = "/profile_api/benchmark/profile";

/// Post account's current profile directly to database. Debug mode must be enabled
/// that route can be used.
#[utoipa::path(
    post,
    path = PATH_POST_PROFILE_TO_DATABASE_BENCHMARK,
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
pub async fn post_profile_to_database_debug_mode_benchmark(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(account_state): Extension<AccountState>,
    Json(profile): Json<ProfileUpdate>,
) -> Result<(), StatusCode> {
    PROFILE.post_profile_to_database_debug_mode_benchmark.incr();
    let old_profile = state.read().profile().profile(account_id).await?;
    let accepted_ages = if account_state != AccountState::InitialSetup {
        state
            .read()
            .profile()
            .accepted_profile_ages(account_id)
            .await?
    } else {
        None
    };
    let profile = profile
        .validate(
            state.config().profile_attributes(),
            &old_profile.profile,
            accepted_ages,
        )
        .into_error_string(DataError::NotAllowed)?;

    if profile.equals_with(&old_profile.profile) {
        return Ok(());
    }

    db_write_multiple!(state, move |cmds| {
        cmds.profile()
            .benchmark_update_profile_bypassing_cache(account_id, profile)
            .await
    })
}

// ------------------- Benchmark routes end ----------------------------

create_open_api_router!(
        fn router_benchmark,
        get_profile_from_database_debug_mode_benchmark,
        post_profile_to_database_debug_mode_benchmark,
);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_BENCHMARK_COUNTERS_LIST,
    get_profile_from_database_debug_mode_benchmark,
    post_profile_to_database_debug_mode_benchmark,
);
