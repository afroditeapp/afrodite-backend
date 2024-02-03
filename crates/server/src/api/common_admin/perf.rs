use axum::{
    extract::{Query, State},
    Extension, Router,
};
use model::Capabilities;
use simple_backend::{app::PerfCounterDataProvider, create_counters};
use simple_backend_model::{PerfHistoryQuery, PerfHistoryQueryResult};

use crate::api::utils::{Json, StatusCode};

pub const PATH_GET_PERF_DATA: &str = "/common_api/perf_data";

/// Get performance data
///
/// # Capabilities
/// Requires admin_server_maintenance_view_info.
#[utoipa::path(
    get,
    path = "/common_api/perf_data",
    params(PerfHistoryQuery),
    responses(
        (status = 200, description = "Get was successfull.", body = PerfHistoryQueryResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_perf_data<S: PerfCounterDataProvider>(
    State(state): State<S>,
    Extension(api_caller_capabilities): Extension<Capabilities>,
    Query(_query): Query<PerfHistoryQuery>,
) -> Result<Json<PerfHistoryQueryResult>, StatusCode> {
    COMMON_ADMIN.get_perf_data.incr();
    if api_caller_capabilities.admin_server_maintenance_view_info {
        let data = state.perf_counter_data().get_history().await;
        Ok(data.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn perf_router(s: crate::app::S) -> Router {
    use axum::routing::get;

    use crate::app::S;

    Router::new()
        .route(PATH_GET_PERF_DATA, get(get_perf_data::<S>))
        .with_state(s)
}

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_PERF_COUNTERS_LIST,
    get_perf_data,
);
