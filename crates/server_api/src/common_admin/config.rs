use std::sync::{
    OnceLock,
    atomic::{AtomicU8, Ordering},
};

use axum::{Extension, extract::State};
use config::bot_config_file::internal::ProfileStringModerationLlmConfigInternal;
use model::{
    AccountIdInternal, AdminBotConfigWarningFlags, BotConfig, BotConfigWarnings,
    DynamicServerConfig, EventToClientInternal, Permissions,
};
use server_data::{
    app::{GetConfig, GetDynamicServerConfig},
    read::GetReadCommandsCommon,
    write::GetWriteCommandsCommon,
};
use server_state::db_write;
use simple_backend::{app::GetManagerApi, create_counters};
use tokio::{
    sync::{Mutex, oneshot},
    time::Duration,
};

use crate::{
    S,
    app::{EventManagerProvider, ReadData, ReadDynamicConfig, WriteData, WriteDynamicConfig},
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
/// * `profile_name_moderation.llm.user_text_template` must contain exactly one `{text}` placeholder.
/// * `profile_text_moderation.llm.user_text_template` must contain exactly one `{text}` placeholder.
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
    let profile_string_moderation_configs = [
        &admin_bot_config.profile_name_moderation,
        &admin_bot_config.profile_text_moderation,
    ];

    for config in profile_string_moderation_configs.iter().map(|v| &v.llm) {
        let count = config
            .user_text_template
            .split(ProfileStringModerationLlmConfigInternal::TEMPLATE_PLACEHOLDER_TEXT)
            .count();

        if count != 2 {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    state.write_config(bot_config).await?;
    Ok(())
}

const PATH_GET_BOT_CONFIG_WARNINGS: &str = "/common_api/bot_config_warnings";

/// Get bot config warnings.
///
/// # Access
/// * [Permissions::admin_server_view_bot_config]
#[utoipa::path(
    get,
    path = PATH_GET_BOT_CONFIG_WARNINGS,
    params(),
    responses(
        (status = 200, description = "Successful.", body = BotConfigWarnings),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_bot_config_warnings(
    State(state): State<S>,
    Extension(api_caller_permissions): Extension<Permissions>,
) -> Result<Json<BotConfigWarnings>, StatusCode> {
    COMMON_ADMIN.get_bot_config_warnings.incr();

    if !api_caller_permissions.admin_server_view_bot_config {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let config = state.read_config().await?;

    if !config.admin_bot {
        return Ok(BotConfigWarnings::default().into());
    }

    if config.remote_bot_login {
        if state
            .manager_api_client()
            .maintenance_status()
            .await
            .admin_bot_offline()
        {
            return Ok(BotConfigWarnings::error_admin_bot_offline().into());
        }

        let admin_bot_id = state
            .read()
            .common_admin()
            .admin_bot_account_ids()
            .await?
            .into_iter()
            .next()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let (request_id, response_receiver) = init_remote_bot_config_warnings_waiter().await?;

        state
            .event_manager()
            .send_connected_event(
                admin_bot_id,
                EventToClientInternal::RequestAdminBotConfigWarnings { request_id },
            )
            .await?;

        let response = tokio::time::timeout(Duration::from_secs(5), response_receiver).await;

        clear_remote_bot_config_warnings_waiter().await;

        let warning_flags = match response {
            Ok(Ok(warning_flags)) => warning_flags,
            Ok(Err(_)) | Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        };

        let config = &config.admin_bot_config;
        let warnings = BotConfigWarnings {
            error: false,
            error_admin_bot_offline: false,
            profile_name_moderation_file_config_missing: config.profile_name_moderation_enabled
                && warning_flags.contains(
                    AdminBotConfigWarningFlags::PROFILE_NAME_MODERATION_FILE_CONFIG_MISSING,
                ),
            profile_text_moderation_file_config_missing: config.profile_text_moderation_enabled
                && warning_flags.contains(
                    AdminBotConfigWarningFlags::PROFILE_TEXT_MODERATION_FILE_CONFIG_MISSING,
                ),
            content_moderation_file_config_missing: config.content_moderation_enabled
                && warning_flags
                    .contains(AdminBotConfigWarningFlags::CONTENT_MODERATION_FILE_CONFIG_MISSING),
            face_verification_file_config_missing: config.face_verification_enabled
                && warning_flags
                    .contains(AdminBotConfigWarningFlags::FACE_VERIFICATION_FILE_CONFIG_MISSING),
            account_verification_file_config_missing: config.account_verification_enabled
                && warning_flags
                    .contains(AdminBotConfigWarningFlags::ACCOUNT_VERIFICATION_FILE_CONFIG_MISSING),
            account_verification_security_content_file_config_missing: config
                .account_verification_enabled
                && config.account_verification.security_content_enabled
                && warning_flags.contains(
                    AdminBotConfigWarningFlags::ACCOUNT_VERIFICATION_SECURITY_CONTENT_FILE_CONFIG_MISSING,
                ),
            report_processing_file_config_missing: config.report_processing_enabled
                && warning_flags
                    .contains(AdminBotConfigWarningFlags::REPORT_PROCESSING_FILE_CONFIG_MISSING),
            report_processing_profile_name_file_config_missing: config
                .report_processing_enabled
                && config.report_processing.profile_name_enabled
                && warning_flags.contains(
                    AdminBotConfigWarningFlags::REPORT_PROCESSING_PROFILE_NAME_FILE_CONFIG_MISSING,
                ),
            report_processing_profile_text_file_config_missing: config
                .report_processing_enabled
                && config.report_processing.profile_text_enabled
                && warning_flags.contains(
                    AdminBotConfigWarningFlags::REPORT_PROCESSING_PROFILE_TEXT_FILE_CONFIG_MISSING,
                ),
            report_processing_profile_content_file_config_missing: config
                .report_processing_enabled
                && config.report_processing.profile_content_enabled
                && warning_flags.contains(
                    AdminBotConfigWarningFlags::REPORT_PROCESSING_PROFILE_CONTENT_FILE_CONFIG_MISSING,
                ),
            report_processing_messages_file_config_missing: config
                .report_processing_enabled
                && config.report_processing.messages_enabled
                && warning_flags.contains(
                    AdminBotConfigWarningFlags::REPORT_PROCESSING_MESSAGES_FILE_CONFIG_MISSING,
                ),
        };

        return Ok(Json(warnings));
    }

    let config = &config.admin_bot_config;
    let bot_config_file = state.config().parsed_files().bot;
    let warnings = BotConfigWarnings {
        error: false,
        error_admin_bot_offline: false,
        profile_name_moderation_file_config_missing: config.profile_name_moderation_enabled
            && bot_config_file.profile_name_moderation.is_none(),
        profile_text_moderation_file_config_missing: config.profile_text_moderation_enabled
            && bot_config_file.profile_text_moderation.is_none(),
        content_moderation_file_config_missing: config.content_moderation_enabled
            && bot_config_file.content_moderation.is_none(),
        face_verification_file_config_missing: config.face_verification_enabled
            && bot_config_file.face_verification.is_none(),
        account_verification_file_config_missing: config.account_verification_enabled
            && bot_config_file.account_verification.is_none(),
        account_verification_security_content_file_config_missing: config
            .account_verification_enabled
            && config.account_verification.security_content_enabled
            && bot_config_file
                .account_verification
                .as_ref()
                .and_then(|v| v.security_content.as_ref())
                .is_none(),
        report_processing_file_config_missing: config.report_processing_enabled
            && bot_config_file.report_processing.is_none(),
        report_processing_profile_name_file_config_missing: config.report_processing_enabled
            && config.report_processing.profile_name_enabled
            && bot_config_file
                .report_processing
                .as_ref()
                .and_then(|v| v.profile_name.as_ref())
                .is_none(),
        report_processing_profile_text_file_config_missing: config.report_processing_enabled
            && config.report_processing.profile_text_enabled
            && bot_config_file
                .report_processing
                .as_ref()
                .and_then(|v| v.profile_text.as_ref())
                .is_none(),
        report_processing_profile_content_file_config_missing: config.report_processing_enabled
            && config.report_processing.profile_content_enabled
            && bot_config_file
                .report_processing
                .as_ref()
                .and_then(|v| v.profile_content.as_ref())
                .is_none(),
        report_processing_messages_file_config_missing: config.report_processing_enabled
            && config.report_processing.messages_enabled
            && bot_config_file
                .report_processing
                .as_ref()
                .and_then(|v| v.messages.as_ref())
                .is_none(),
    };

    Ok(Json(warnings))
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

static REMOTE_BOT_CONFIG_WARNINGS_WAITER: OnceLock<Mutex<Option<RemoteBotConfigWarningsWaiter>>> =
    OnceLock::new();

struct RemoteBotConfigWarningsWaiter {
    request_id: u8,
    sender: oneshot::Sender<AdminBotConfigWarningFlags>,
}

static REMOTE_BOT_CONFIG_WARNINGS_REQUEST_ID: AtomicU8 = AtomicU8::new(0);

fn next_remote_bot_config_warnings_request_id() -> u8 {
    REMOTE_BOT_CONFIG_WARNINGS_REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

async fn init_remote_bot_config_warnings_waiter()
-> Result<(u8, oneshot::Receiver<AdminBotConfigWarningFlags>), StatusCode> {
    let request_id = next_remote_bot_config_warnings_request_id();
    let (sender, receiver) = oneshot::channel();
    let waiters = REMOTE_BOT_CONFIG_WARNINGS_WAITER.get_or_init(|| Mutex::new(None));
    // Replace previous waiter to keep the latest request active.
    *waiters.lock().await = Some(RemoteBotConfigWarningsWaiter { request_id, sender });

    Ok((request_id, receiver))
}

async fn clear_remote_bot_config_warnings_waiter() {
    let waiters = REMOTE_BOT_CONFIG_WARNINGS_WAITER.get_or_init(|| Mutex::new(None));
    let mut waiters = waiters.lock().await;
    *waiters = None;
}

pub(crate) async fn complete_remote_bot_config_warnings_waiter(
    request_id: u8,
    warning_flags: AdminBotConfigWarningFlags,
) {
    let waiters = REMOTE_BOT_CONFIG_WARNINGS_WAITER.get_or_init(|| Mutex::new(None));
    let mut waiters = waiters.lock().await;

    if waiters
        .as_ref()
        .is_some_and(|waiter| waiter.request_id == request_id)
        && let Some(waiter) = waiters.take()
    {
        let _ = waiter.sender.send(warning_flags);
    }
}

create_open_api_router!(
    fn router_config,
    get_bot_config,
    post_bot_config,
    get_bot_config_warnings,
    get_dynamic_server_config,
    post_dynamic_server_config,
);

create_counters!(
    CommonAdminCounters,
    COMMON_ADMIN,
    COMMON_ADMIN_CONFIG_COUNTERS_LIST,
    get_bot_config,
    post_bot_config,
    get_bot_config_warnings,
    get_dynamic_server_config,
    post_dynamic_server_config,
);
