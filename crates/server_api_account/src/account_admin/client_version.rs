use axum::{Extension, extract::State};
use model::Permissions;
use model_account::{GetClientVersionStatisticsResult, GetClientVersionStatisticsSettings};
use server_api::{S, create_open_api_router};
use server_data_account::read::GetReadCommandsAccount;
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    utils::{Json, StatusCode},
};

const PATH_POST_GET_CLIENT_VERSION_STATISTICS: &str = "/account_api/client_version_statistics";

/// Get client version statistics.
///
/// HTTP method is POST to allow JSON request body.
///
/// # Permissions
/// Requires admin_server_maintenance_view_info.
#[utoipa::path(
    post,
    path = PATH_POST_GET_CLIENT_VERSION_STATISTICS,
    request_body = GetClientVersionStatisticsSettings,
    responses(
        (status = 200, description = "Successfull.", body = GetClientVersionStatisticsResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_client_version_statistics(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(settings): Json<GetClientVersionStatisticsSettings>,
) -> Result<Json<GetClientVersionStatisticsResult>, StatusCode> {
    ACCOUNT_ADMIN.post_get_client_version_statistics.incr();
    if api_caller_permissions.admin_server_maintenance_view_info {
        let data = state
            .read()
            .account_admin_history()
            .get_client_version_statistics(settings)
            .await?;
        Ok(data.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

create_open_api_router!(fn router_admin_client_version, post_get_client_version_statistics,);

create_counters!(
    AccountAdminCounters,
    ACCOUNT_ADMIN,
    ACCOUNT_ADMIN_CLIENT_VERSION_PERF_COUNTERS_LIST,
    post_get_client_version_statistics,
);
