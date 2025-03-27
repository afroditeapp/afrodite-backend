use axum::{
    extract::{Query, State},
    Extension,
};
use model::{GetApiUsageStatisticsResult, GetApiUsageStatisticsSettings, Permissions};
use simple_backend::{app::PerfCounterDataProvider, create_counters};
use simple_backend_model::{PerfMetricQuery, PerfMetricQueryResult};

use server_common::app::GetAccounts;
use server_data::{app::ReadData, read::GetReadCommandsCommon};

use crate::{
    create_open_api_router,
    utils::{Json, StatusCode},
    S,
};

// TODO(prod): Check that does PerfMetricQuery value deserialization work
//             with when making API requests with generated API code.

const PATH_GET_PERF_DATA: &str = "/common_api/perf_data";

/// Get performance data
///
/// # Permissions
/// Requires admin_server_maintenance_view_info.
#[utoipa::path(
    get,
    path = PATH_GET_PERF_DATA,
    params(PerfMetricQuery),
    responses(
        (status = 200, description = "Get was successfull.", body = PerfMetricQueryResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_perf_data(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Query(_query): Query<PerfMetricQuery>,
) -> Result<Json<PerfMetricQueryResult>, StatusCode> {
    COMMON_ADMIN.get_perf_data.incr();
    if api_caller_permissions.admin_server_maintenance_view_info {
        let data = state.perf_counter_data().get_history(false).await;
        Ok(data.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_GET_API_USAGE_DATA: &str = "/common_api/api_usage_data";

/// Get API usage data for account
///
/// HTTP method is POST because JSON request body requires it.
///
/// # Permissions
/// Requires [Permissions::admin_view_private_info].
#[utoipa::path(
    post,
    path = PATH_POST_GET_API_USAGE_DATA,
    request_body = GetApiUsageStatisticsSettings,
    responses(
        (status = 200, description = "Successful.", body = GetApiUsageStatisticsResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_api_usage_data(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(settings): Json<GetApiUsageStatisticsSettings>,
) -> Result<Json<GetApiUsageStatisticsResult>, StatusCode> {
    COMMON_ADMIN.post_get_api_usage_data.incr();

    if !api_caller_permissions.admin_view_private_info {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let requested_account = state.get_internal_id(settings.account).await?;

    let data = state
        .read()
        .common_admin()
        .api_usage()
        .get_api_usage_statistics(requested_account, settings)
        .await?;

    Ok(data.into())
}

create_open_api_router!(fn router_statistics, get_perf_data, post_get_api_usage_data,);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_STATISTICS_COUNTERS_LIST,
    get_perf_data,
    post_get_api_usage_data,
);
