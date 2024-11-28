use axum::{
    extract::{Query, State},
    Extension,
};
use model_profile::{
    GetProfileStatisticsHistoryParams, GetProfileStatisticsHistoryResult, Permissions, ProfileStatisticsHistoryValueTypeInternal
};
use obfuscate_api_macro::obfuscate_api;
use server_api::S;
use server_api::create_open_api_router;
use server_data_profile::read::GetReadProfileCommands;
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::{
        ReadData,
    },
    utils::{Json, StatusCode},
};

#[obfuscate_api]
const PATH_GET_PROFILE_STATISTICS_HISTORY: &str = "/profile_api/profile_statistics_history";

#[utoipa::path(
    get,
    path = PATH_GET_PROFILE_STATISTICS_HISTORY,
    params(GetProfileStatisticsHistoryParams),
    responses(
        (status = 200, description = "Successful", body = GetProfileStatisticsHistoryResult),
        (status = 401, description = "Unauthorized"),
        (
            status = 500,
            description = "Internal server error",
        ),
    ),
    security(("access_token" = [])),
)]
pub async fn get_profile_statistics_history(
    State(state): State<S>,
    Extension(permissions): Extension<Permissions>,
    Query(params): Query<GetProfileStatisticsHistoryParams>,
) -> Result<Json<GetProfileStatisticsHistoryResult>, StatusCode> {
    PROFILE.get_profile_statistics_history.incr();

    if !permissions.admin_profile_statistics {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let p: ProfileStatisticsHistoryValueTypeInternal = params.try_into()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let r = state.read().profile_admin_history().statistics().profile_statistics(p).await?;

    Ok(r.into())
}

pub fn admin_statistics_router(
    s: S,
) -> OpenApiRouter {
    create_open_api_router!(
        s,
        get_profile_statistics_history,
    )
}

create_counters!(
    ProfileCounters,
    PROFILE,
    PROFILE_ADMIN_STATISTICS_COUNTERS_LIST,
    get_profile_statistics_history,
);
