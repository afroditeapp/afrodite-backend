use axum::{extract::State, Extension};
use model::{AccountIdInternal, ClientConfig, ClientFeaturesFileHash, CustomReportsFileHash};
use server_data::{app::GetConfig, read::GetReadCommandsCommon};
use simple_backend::create_counters;

use crate::{
    app::ReadData,
    create_open_api_router,
    utils::{Json, StatusCode},
    S,
};

const PATH_GET_CLIENT_CONFIG: &str = "/common_api/client_config";

#[utoipa::path(
    get,
    path = PATH_GET_CLIENT_CONFIG,
    responses(
        (status = 200, description = "Get successfull.", body = ClientConfig),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_client_config(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<ClientConfig>, StatusCode> {
    COMMON.get_client_config.incr();
    let sync_version = state.read().common().client_config_sync_version(account_id).await?;
    let info = ClientConfig {
        client_features: state.config().client_features_sha256().map(|v| ClientFeaturesFileHash::new(v.to_string())),
        custom_reports: state.config().custom_reports_sha256().map(|v| CustomReportsFileHash::new(v.to_string())),
        profile_attributes: state.config().profile_attributes().map(|a| a.info_for_client()).cloned(),
        sync_version,
    };
    Ok(info.into())
}

create_open_api_router!(fn router_client_config, get_client_config,);

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_CLIENT_CONFIG_COUNTERS_LIST,
    get_client_config,
);
