use axum::{Extension, extract::State};
use config::bot_config_file::internal::ProfileStringModerationLlmConfigInternal;
use model::{AccountIdInternal, BotConfig, DynamicServerConfig, Permissions};
use server_data::{
    app::GetDynamicServerConfig, read::GetReadCommandsCommon, write::GetWriteCommandsCommon,
};
use server_state::db_write;
use simple_backend::create_counters;

use crate::{
    S,
    app::{ReadData, ReadDynamicConfig, WriteData, WriteDynamicConfig},
    create_open_api_router,
    utils::{Json, StatusCode},
};

const PATH_GET_BOT_CONFIG: &str = "/common_api/bot_config";

/// Get bot config.
///
/// # Access
/// * [Permissions::admin_server_view_bot_config]
/// * Bot account
#[utoipa::path(
    get,
    path = PATH_GET_BOT_CONFIG,
    responses(
        (status = 200, description = "Get was successfull.", body = BotConfig),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_bot_config(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Extension(api_caller_id): Extension<AccountIdInternal>,
) -> Result<Json<BotConfig>, StatusCode> {
    COMMON_ADMIN.get_bot_config.incr();

    let is_bot = state.read().common().is_bot(api_caller_id).await?;

    if api_caller_permissions.admin_server_view_bot_config || is_bot {
        let config = state.read_config().await?;
        Ok(config.into())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

const PATH_POST_BOT_CONFIG: &str = "/common_api/bot_config";

/// Save bot config.
///
/// # Validation
/// The following fields must contain exactly one `{text}` placeholder:
/// * `profile_name_moderation.llm.user_text_template`
/// * `profile_text_moderation.llm.user_text_template`
/// * `report_processing.profile_name.llm.user_text_template`
/// * `report_processing.profile_text.llm.user_text_template`
/// * `report_processing.messages.llm.user_text_template`
/// * `report_processing.messages.llm.report_creator_message_template`
/// * `report_processing.messages.llm.report_target_message_template`
///
/// # Access
/// * [Permissions::admin_server_edit_bot_config]
#[utoipa::path(
    post,
    path = PATH_POST_BOT_CONFIG,
    params(),
    request_body(content = BotConfig),
    responses(
        (status = 200, description = "Save was successfull."),
        (status = 400, description = "Invalid configuration."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_bot_config(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(bot_config): Json<BotConfig>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_bot_config.incr();

    if !api_caller_permissions.admin_server_edit_bot_config {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let admin_bot_config = &bot_config.admin_bot_config;

    let template_strings: [&str; 7] = [
        &admin_bot_config
            .profile_name_moderation
            .llm
            .user_text_template,
        &admin_bot_config
            .profile_text_moderation
            .llm
            .user_text_template,
        &admin_bot_config
            .report_processing
            .profile_name
            .llm
            .base
            .user_text_template,
        &admin_bot_config
            .report_processing
            .profile_text
            .llm
            .base
            .user_text_template,
        &admin_bot_config
            .report_processing
            .messages
            .llm
            .base
            .user_text_template,
        &admin_bot_config
            .report_processing
            .messages
            .llm
            .report_creator_message_template,
        &admin_bot_config
            .report_processing
            .messages
            .llm
            .report_target_message_template,
    ];

    for template in &template_strings {
        let count = template
            .split(ProfileStringModerationLlmConfigInternal::TEMPLATE_PLACEHOLDER_TEXT)
            .count();

        if count != 2 {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    state.write_config(bot_config).await?;
    Ok(())
}

const PATH_GET_DYNAMIC_SERVER_CONFIG: &str = "/common_api/dynamic_server_config";

/// Get server config.
///
/// # Access
/// * [Permissions::admin_server_view_server_config]
#[utoipa::path(
    get,
    path = PATH_GET_DYNAMIC_SERVER_CONFIG,
    responses(
        (status = 200, description = "Successful.", body = DynamicServerConfig),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_dynamic_server_config(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<DynamicServerConfig>, StatusCode> {
    COMMON_ADMIN.get_dynamic_server_config.incr();

    if !api_caller_permissions.admin_server_view_server_config {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let config = state
        .dynamic_server_config_manager()
        .dynamic_server_config()
        .await
        .unwrap_or_default();

    Ok(config.into())
}

const PATH_POST_DYNAMIC_SERVER_CONFIG: &str = "/common_api/dynamic_server_config";

/// Save server config.
///
/// # Access
/// * [Permissions::admin_server_edit_server_config]
#[utoipa::path(
    post,
    path = PATH_POST_DYNAMIC_SERVER_CONFIG,
    request_body(content = DynamicServerConfig),
    responses(
        (status = 200, description = "Successful."),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn post_dynamic_server_config(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
    Json(config): Json<DynamicServerConfig>,
) -> Result<(), StatusCode> {
    COMMON_ADMIN.post_dynamic_server_config.incr();

    if !api_caller_permissions.admin_server_edit_server_config {
        return Err(StatusCode::UNAUTHORIZED);
    }

    db_write!(state, move |cmds| {
        cmds.common()
            .client_config()
            .upsert_dynamic_server_config(config)
            .await
    })?;

    Ok(())
}

create_open_api_router!(
    fn router_config,
    get_bot_config,
    post_bot_config,
    get_dynamic_server_config,
    post_dynamic_server_config,
);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_CONFIG_COUNTERS_LIST,
    get_bot_config,
    post_bot_config,
    get_dynamic_server_config,
    post_dynamic_server_config,
);
