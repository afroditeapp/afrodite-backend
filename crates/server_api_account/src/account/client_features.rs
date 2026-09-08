use axum::extract::State;
use model::{
    ClientFeaturesConfigHash, DynamicClientFeaturesConfigHash, EmailRegistrationPlatforms,
};
use model_account::{GetClientFeaturesConfigResult, GetDynamicClientFeaturesConfigResult};
use server_api::{S, app::GetConfig, create_open_api_router};
use server_data::app::{GetDynamicClientFeatures, GetDynamicServerConfig};
use simple_backend::create_counters;

use crate::utils::{Json, StatusCode};

const PATH_POST_GET_CLIENT_FEATURES_CONFIG: &str = "/account_api/client_features_config";

#[utoipa::path(
    post,
    path = PATH_POST_GET_CLIENT_FEATURES_CONFIG,
    request_body = ClientFeaturesConfigHash,
    responses(
        (status = 200, description = "Successful.", body = GetClientFeaturesConfigResult),
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
        (status = 200, description = "Successful.", body = GetDynamicClientFeaturesConfigResult),
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

pub const PATH_GET_EMAIL_REGISTRATION_PLATFORMS: &str = "/account_api/email_registration_platforms";

/// Get email registration platforms from dynamic server config.
///
/// This route is unauthenticated so that the client can check whether
/// email registration is enabled for its platform before starting
/// the email registration flow.
#[utoipa::path(
    get,
    path = PATH_GET_EMAIL_REGISTRATION_PLATFORMS,
    security(),
    responses(
        (status = 200, description = "Successful.", body = EmailRegistrationPlatforms),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn get_email_registration_platforms(
    State(state): State<S>,
) -> Result<Json<EmailRegistrationPlatforms>, StatusCode> {
    ACCOUNT.get_email_registration_platforms.incr();

    let config = state
        .dynamic_server_config_manager()
        .dynamic_server_config_ref()
        .await;

    Ok(config
        .as_ref()
        .map(|config| config.email_registration_platforms.clone())
        .unwrap_or_default()
        .into())
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
    get_email_registration_platforms,
);
