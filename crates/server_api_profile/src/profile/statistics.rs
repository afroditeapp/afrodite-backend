use axum::{
    extract::{Query, State},
    Extension,
};
use model::{
    GetProfileStatisticsParams, GetProfileStatisticsResult, Permissions
};
use obfuscate_api_macro::obfuscate_api;
use server_api::create_open_api_router;
use server_data_profile::{app::ProfileStatisticsCacheProvider, read::GetReadProfileCommands};
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::{
        ReadData, StateBase,
    },
    utils::{Json, StatusCode},
};

#[obfuscate_api]
const PATH_GET_PROFILE_STATISTICS: &str = "/profile_api/profile_statistics";

/// Non default values for [model::GetProfileStatisticsParams]
/// requires [model::Permissions::admin_profile_statistics].
#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_STATISTICS,
    params(GetProfileStatisticsParams),
    responses(
        (status = 200, description = "Successful", body = GetProfileStatisticsResult),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_statistics<
    S: ReadData + ProfileStatisticsCacheProvider,
>(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Query(params): Query<GetProfileStatisticsParams>,
) -> Result<Json<GetProfileStatisticsResult>, StatusCode> {
    PROFILE.get_profile_statistics.incr();

    if !permissions.admin_profile_statistics && params != GetProfileStatisticsParams::default() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = if params == GetProfileStatisticsParams::default() {
        state.profile_statistics_cache().get_or_update_statistics(&state).await?
    } else {
        state.read().profile().statistics().profile_statistics(params.profile_visibility).await?
    };

    Ok(r.into())
}

pub fn statistics_router<
    S: StateBase + ReadData + ProfileStatisticsCacheProvider,
>(
    s: S,
) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_profile_statistics::<S>,
    )
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_STATISTICS_COUNTERS_LIST,
    get_profile_statistics,
);
