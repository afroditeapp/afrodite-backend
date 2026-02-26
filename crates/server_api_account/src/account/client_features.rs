use axum::extract::State;
use model::{ClientFeaturesConfigHash, DynamicClientFeaturesConfigHash};
use model_account::{GetClientFeaturesConfigResult, GetDynamicClientFeaturesConfigResult};
use server_api::{S, app::GetConfig, create_open_api_router};
use server_data::app::GetDynamicClientFeatures;
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

const PATH_POST_GET_CLIENT_FEATURES_CONFIG: &str = "/account_api/client_features_config";

#[utoipa::path(
    post,
    path = PATH_POST_GET_CLIENT_FEATURES_CONFIG,
    request_body = ClientFeaturesConfigHash,
    responses(
        (status = 200, description = "Successfull.", body = GetClientFeaturesConfigResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_client_features_config(
    State(state): State<S>,
    Json(requested_hash): Json<ClientFeaturesConfigHash>,
) -> Result<Json<GetClientFeaturesConfigResult>, StatusCode> {
    ACCOUNT.post_get_client_features_config.incr();

    let r = if requested_hash.hash() == state.config().client_features_sha256() {
        GetClientFeaturesConfigResult {
            config: Some(state.config().client_features().clone()),
        }
    } else {
        GetClientFeaturesConfigResult { config: None }
    };

    Ok(r.into())
}

const PATH_POST_GET_DYNAMIC_CLIENT_FEATURES_CONFIG: &str =
    "/account_api/dynamic_client_features_config";

#[utoipa::path(
    post,
    path = PATH_POST_GET_DYNAMIC_CLIENT_FEATURES_CONFIG,
    request_body = DynamicClientFeaturesConfigHash,
    responses(
        (status = 200, description = "Successfull.", body = GetDynamicClientFeaturesConfigResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_get_dynamic_client_features_config(
    State(state): State<S>,
    Json(requested_hash): Json<DynamicClientFeaturesConfigHash>,
) -> Result<Json<GetDynamicClientFeaturesConfigResult>, StatusCode> {
    ACCOUNT.post_get_dynamic_client_features_config.incr();

    let current = state
        .dynamic_client_features_manager()
        .dynamic_client_features()
        .await;

    let r = if current
        .as_ref()
        .map(|v| requested_hash.hash() == v.hash.hash())
        .unwrap_or(false)
    {
        GetDynamicClientFeaturesConfigResult {
            config: current.map(|v| v.config),
        }
    } else {
        GetDynamicClientFeaturesConfigResult { config: None }
    };

    Ok(r.into())
}

create_open_api_router!(
        fn router_client_features,
        post_get_client_features_config,
        post_get_dynamic_client_features_config,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_CLIENT_FEATURES_COUNTERS_LIST,
    post_get_client_features_config,
    post_get_dynamic_client_features_config,
);
