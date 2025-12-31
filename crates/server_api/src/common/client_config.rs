use axum::{Extension, extract::State};
use model::{
    AccountIdInternal, ClientConfig, ClientFeaturesConfigHash, ClientLanguage,
    CustomReportsConfigHash, GetClientLanguage,
};
use server_data::{app::GetConfig, read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use server_state::db_write;
use simple_backend::create_counters;

use crate::{
    S,
    app::{ReadData, WriteData},
    create_open_api_router,
    utils::{Json, StatusCode},
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
    let sync_version = state
        .read()
        .common()
        .client_config()
        .client_config_sync_version(account_id)
        .await?;
    let info = ClientConfig {
        client_features: Some(ClientFeaturesConfigHash::new(
            state.config().client_features_sha256().to_string(),
        )),
        custom_reports: Some(CustomReportsConfigHash::new(
            state.config().custom_reports_sha256().to_string(),
        )),
        profile_attributes: Some(
            state
                .config()
                .profile_attributes()
                .config_for_client()
                .clone(),
        ),
        sync_version,
    };
    Ok(info.into())
}

const PATH_GET_CLIENT_LANGUAGE: &str = "/common_api/client_language";

#[utoipa::path(
    get,
    path = PATH_GET_CLIENT_LANGUAGE,
    responses(
        (status = 200, description = "Successfull.", body = GetClientLanguage),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_client_language(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
) -> Result<Json<GetClientLanguage>, StatusCode> {
    COMMON.get_client_language.incr();
    let value = state
        .read()
        .common()
        .client_config()
        .client_language(account_id)
        .await?;
    let value = GetClientLanguage { l: value };
    Ok(value.into())
}

const PATH_POST_CLIENT_LANGUAGE: &str = "/common_api/client_language";

#[utoipa::path(
    post,
    path = PATH_POST_CLIENT_LANGUAGE,
    request_body = ClientLanguage,
    responses(
        (status = 200, description = "Successfull."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_client_language(
    State(state): State<S>,
    Extension(account_id): Extension<AccountIdInternal>,
    Json(value): Json<ClientLanguage>,
) -> Result<(), StatusCode> {
    COMMON.post_client_language.incr();
    db_write!(state, move |cmds| {
        cmds.common()
            .client_config()
            .client_language(account_id, value)
            .await
    })?;
    Ok(())
}

create_open_api_router!(fn router_client_config, get_client_config, get_client_language, post_client_language,);

create_counters!(
    CommonCounters,
    COMMON,
    COMMON_CLIENT_CONFIG_COUNTERS_LIST,
    get_client_config,
    get_client_language,
    post_client_language,
);
