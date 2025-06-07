use axum::{
    extract::{Query, State},
    Extension,
};
use model_profile::{GetProfileStatisticsParams, GetProfileStatisticsResult, Permissions};
use server_api::{app::ProfileStatisticsCacheProvider, create_open_api_router, S};
use server_data_profile::{read::GetReadProfileCommands, statistics::ProfileStatisticsCacheUtils};
use simple_backend::{create_counters, app::PerfCounterDataProvider};

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

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
pub async fn get_profile_statistics(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Query(params): Query<GetProfileStatisticsParams>,
) -> Result<Json<GetProfileStatisticsResult>, StatusCode> {
    PROFILE.get_profile_statistics.incr();

    if !permissions.admin_profile_statistics && params.contains_admin_settings() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r: GetProfileStatisticsResult = if params.contains_admin_settings() {
        state
            .read()
            .profile()
            .statistics()
            .profile_statistics(params.profile_visibility.unwrap_or_default(), state.perf_counter_data_arc())
            .await?
            .into()
    } else {
        state
            .profile_statistics_cache()
            .get_or_update_statistics(state.read(), state.perf_counter_data_arc())
            .await?
            .into()
    };

    Ok(r.into())
}

create_open_api_router!(fn router_statistics, get_profile_statistics,);

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_STATISTICS_COUNTERS_LIST,
    get_profile_statistics,
);
