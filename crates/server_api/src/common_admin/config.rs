use axum::{extract::State, Extension};
use model::{AccountIdInternal, BackendConfig, Permissions};
use simple_backend::create_counters;
use tracing::info;

use crate::{
    app::{ReadDynamicConfig, WriteDynamicConfig},
    create_open_api_router,
    utils::{Json, StatusCode},
    S,
};

const PATH_GET_BACKEND_CONFIG: &str = "/common_api/backend_config";

/// Get dynamic backend config.
///
/// # Permissions
/// Requires admin_server_maintenance_view_backend_settings.
#[utoipa::path(
    get,
    path = PATH_GET_BACKEND_CONFIG,
    params(),
    responses(
        (status = 200, description = "Get was successfull.", body = BackendConfig),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_backend_config(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<BackendConfig>, StatusCode> {
    COMMON_ADMIN.get_backend_config.incr();

    if api_caller_permissions.admin_server_maintenance_view_backend_config {
        let config = state.read_config().await?;
        Ok(config.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_BACKEND_CONFIG: &str = "/common_api/backend_config";

/// Save dynamic backend config.
///
/// # Permissions
/// Requires admin_server_maintenance_save_backend_settings.
#[utoipa::path(
    post,
    path = PATH_POST_BACKEND_CONFIG,
    params(),
    request_body(content = BackendConfig),
    responses(
        (status = 200, description = "Save was successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_backend_config(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(backend_config): Json<BackendConfig>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_backend_config.incr();

    if api_caller_permissions.admin_server_maintenance_save_backend_config {
        info!(
            "Saving dynamic backend config, account: {}, settings: {:#?}",
            api_caller_account_id.as_id(),
            backend_config
        );
        state.write_config(backend_config).await?;

        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

create_open_api_router!(fn router_config, get_backend_config, post_backend_config,);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_CONFIG_COUNTERS_LIST,
    get_backend_config,
    post_backend_config,
);
