use axum::{
    extract::State,
    Extension,
};
use model::{GetApiUsageStatisticsResult, GetApiUsageStatisticsSettings, GetIpAddressStatisticsResult, GetIpAddressStatisticsSettings, Permissions};
use simple_backend::{app::PerfCounterDataProvider, create_counters};
use simple_backend_model::{PerfMetricQuery, PerfMetricQueryResult};

use server_common::app::GetAccounts;
use server_data::{app::ReadData, read::GetReadCommandsCommon};

use crate::{
    create_open_api_router,
    utils::{Json, StatusCode},
    S,
};

const PATH_POST_GET_PERF_DATA: &str = "/common_api/perf_data";

/// Get performance data
///
/// HTTP method is POST because JSON request body requires it.
///
/// # Permissions
/// Requires admin_server_maintenance_view_info.
#[utoipa::path(
    post,
    path = PATH_POST_GET_PERF_DATA,
    request_body = PerfMetricQuery,
    responses(
        (status = 200, description = "Successful.", body = PerfMetricQueryResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_perf_data(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(_query): Json<PerfMetricQuery>,
) -> Result<Json<PerfMetricQueryResult>, StatusCode> {
    COMMON_ADMIN.post_get_perf_data.incr();
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
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let requested_account = state.get_internal_id(settings.account).await?;

    let data = state
        .read()
        .common_admin()
        .statistics()
        .get_api_usage_statistics(requested_account, settings)
        .await?;

    Ok(data.into())
}

const PATH_POST_GET_IP_ADDRESS_USAGE_DATA: &str = "/common_api/ip_address_usage_data";

/// Get IP address usage data for account
///
/// HTTP method is POST because JSON request body requires it.
///
/// # Permissions
/// Requires [Permissions::admin_view_private_info].
#[utoipa::path(
    post,
    path = PATH_POST_GET_IP_ADDRESS_USAGE_DATA,
    request_body = GetIpAddressStatisticsSettings,
    responses(
        (status = 200, description = "Successful.", body = GetIpAddressStatisticsResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_ip_address_usage_data(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(settings): Json<GetIpAddressStatisticsSettings>,
) -> Result<Json<GetIpAddressStatisticsResult>, StatusCode> {
    COMMON_ADMIN.post_get_ip_address_usage_data.incr();

    if !api_caller_permissions.admin_view_private_info {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let requested_account = state.get_internal_id(settings.account).await?;

    let data = state
        .read()
        .common_admin()
        .statistics()
        .get_ip_address_statistics(requested_account)
        .await?;

    Ok(data.into())
}

create_open_api_router!(fn router_statistics, post_get_perf_data, post_get_api_usage_data, post_get_ip_address_usage_data,);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_STATISTICS_COUNTERS_LIST,
    post_get_perf_data,
    post_get_api_usage_data,
    post_get_ip_address_usage_data,
);
