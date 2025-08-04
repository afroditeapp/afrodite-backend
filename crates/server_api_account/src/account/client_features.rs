use axum::extract::State;
use model::ClientFeaturesConfigHash;
use model_account::GetClientFeaturesConfigResult;
use server_api::{S, app::GetConfig, create_open_api_router};
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

    let r = if Some(requested_hash.hash()) == state.config().client_features_sha256() {
        GetClientFeaturesConfigResult {
            config: state.config().client_features().cloned(),
        }
    } else {
        GetClientFeaturesConfigResult { config: None }
    };

    Ok(r.into())
}

create_open_api_router!(
        fn router_client_features,
        post_get_client_features_config,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_CLIENT_FEATURES_COUNTERS_LIST,
    post_get_client_features_config,
);
