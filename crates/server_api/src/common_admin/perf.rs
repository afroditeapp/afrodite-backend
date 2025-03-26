use axum::{
    extract::{Query, State},
    Extension,
};
use model::Permissions;
use simple_backend::{app::PerfCounterDataProvider, create_counters};
use simple_backend_model::{PerfMetricQuery, PerfMetricQueryResult};

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

create_open_api_router!(fn router_perf, get_perf_data,);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_PERF_COUNTERS_LIST,
    get_perf_data,
);
