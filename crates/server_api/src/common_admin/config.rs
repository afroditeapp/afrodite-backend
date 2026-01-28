use axum::{Extension, extract::State};
use config::bot_config_file::internal::LlmStringModerationConfig;
use model::{AccountIdInternal, BackendConfig, Permissions};
use server_data::read::GetReadCommandsCommon;
use simple_backend::create_counters;

use crate::{
    S,
    app::{ReadData, ReadDynamicConfig, WriteDynamicConfig},
    create_open_api_router,
    utils::{Json, StatusCode},
};

const PATH_GET_BACKEND_CONFIG: &str = "/common_api/backend_config";

/// Get dynamic backend config.
///
/// # Access
/// * [Permissions::admin_server_maintenance_view_backend_config]
/// * Bot account
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
    Extension(api_caller_id): Extension<AccountIdInternal>,
) -> Result<Json<BackendConfig>, StatusCode> {
    COMMON_ADMIN.get_backend_config.incr();

    let is_bot = state.read().common().is_bot(api_caller_id).await?;

    if api_caller_permissions.admin_server_maintenance_view_backend_config || is_bot {
        let config = state.read_config().await?;
        Ok(config.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_BACKEND_CONFIG: &str = "/common_api/backend_config";

/// Save dynamic backend config.
///
/// # Validation
/// * `profile_name_moderation.llm.user_text_template` must contain exactly one `{text}` placeholder.
/// * `profile_text_moderation.llm.user_text_template` must contain exactly one `{text}` placeholder.
///
/// # Access
/// * [Permissions::admin_server_maintenance_save_backend_config]
#[utoipa::path(
    post,
    path = PATH_POST_BACKEND_CONFIG,
    params(),
    request_body(content = BackendConfig),
    responses(
        (status = 200, description = "Save was successfull."),
        (status = 400, description = "Invalid configuration."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_backend_config(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(backend_config): Json<BackendConfig>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_backend_config.incr();

    if !api_caller_permissions.admin_server_maintenance_save_backend_config {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if let Some(admin_bot_config) = &backend_config.admin_bot_config {
        let profile_string_moderation_configs = [
            admin_bot_config.profile_name_moderation.as_ref(),
            admin_bot_config.profile_text_moderation.as_ref(),
        ];

        for config in profile_string_moderation_configs
            .iter()
            .flatten()
            .flat_map(|v| v.llm.as_ref())
        {
            let count = config
                .user_text_template
                .split(LlmStringModerationConfig::TEMPLATE_PLACEHOLDER_TEXT)
                .count();

            if count != 2 {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    state.write_config(backend_config).await?;
    Ok(())
}

create_open_api_router!(fn router_config, get_backend_config, post_backend_config,);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_CONFIG_COUNTERS_LIST,
    get_backend_config,
    post_backend_config,
);
