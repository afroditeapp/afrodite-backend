use axum::{
    extract::{Query, State},
    Extension,
};
use model::{
    AccountIdInternal, GetProfileStatisticsParams, GetProfileStatisticsResult, Permissions
};
use obfuscate_api_macro::obfuscate_api;
use server_api::{create_open_api_router, db_write_multiple, result::WrappedContextExt};
use server_data::read::GetReadCommandsCommon;
use server_data_profile::{read::GetReadProfileCommands, write::GetWriteCommandsProfile};
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
    S: ReadData,
>(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Extension(permissions): Extension<Permissions>,
    Query(params): Query<GetProfileStatisticsParams>,
) -> Result<Json<GetProfileStatisticsResult>, StatusCode> {
    PROFILE.get_profile_statistics.incr();

    if !permissions.admin_profile_statistics && params != GetProfileStatisticsParams::default() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    unimplemented!()
}

pub fn statistics_router<
    S: StateBase + ReadData,
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
