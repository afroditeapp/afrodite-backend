use std::net::SocketAddr;

use axum::extract::{ConnectInfo, State};
use model::ClientType;
use model_account::{
    AccountId, BotAccount, EmailAddress, GetBotsResult, LoginResult, RemoteBotLogin,
    RemoteBotPassword, SignInWithInfo,
};
use server_api::{
    S,
    app::{GetConfig, ReadDynamicConfig},
    common::is_ip_address_accepted,
    db_write,
};
use server_data::{read::GetReadCommandsCommon, write::GetWriteCommandsCommon};
use server_data_account::{read::GetReadCommandsAccount, write::GetWriteCommandsAccount};
use simple_backend::create_counters;

use super::account::login_impl;
use crate::{
    app::{GetAccounts, ReadData, WriteData},
    utils::{Json, StatusCode},
};

pub const PATH_BOT_LOGIN: &str = "/account_api/bot_login";

/// Get new AccessToken for a bot account. If the account is not registered
/// as a bot account, then the request will fail.
///
/// Available only from local bot API port.
#[utoipa::path(
    post,
    path = PATH_BOT_LOGIN,
    security(),
    request_body = AccountId,
    responses(
        (status = 200, description = "Login successful.", body = LoginResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_bot_login(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Json(id): Json<AccountId>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT_BOT.post_bot_login.incr();

    let internal_id = state.get_internal_id(id).await?;
    let is_bot = state.read().account().is_bot_account(internal_id).await?;
    if !is_bot {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = login_impl(id, address, &state).await?;

    if let Some(aid) = r.aid() {
        // Login successful
        let id = state.get_internal_id(aid).await?;
        db_write!(state, move |cmds| {
            cmds.common()
                .client_config()
                .client_login_session_platform(id, ClientType::Bot)
                .await
        })?;
    }

    Ok(r.into())
}

pub const PATH_BOT_REGISTER: &str = "/account_api/bot_register";

/// Register a new bot account. Returns new account ID which is UUID.
///
/// Available only from local bot API port.
#[utoipa::path(
    post,
    path = PATH_BOT_REGISTER,
    security(),
    responses(
        (status = 200, description = "New profile created.", body = AccountId),
        (status = 500, description = "Internal server error."),
    )
)]
pub async fn post_bot_register(State(state): State<S>) -> Result<Json<AccountId>, StatusCode> {
    ACCOUNT_BOT.post_bot_register.incr();
    let new_account_id = state
        .data_all_access()
        .register_impl(SignInWithInfo::default(), None)
        .await?;

    db_write!(state, move |cmds| {
        cmds.common().set_is_bot_account(new_account_id, true).await
    })?;

    Ok(new_account_id.as_id().into())
}

pub const PATH_GET_BOTS: &str = "/account_api/get_bots";

/// Get admin and user bot accounts by email pattern.
/// Admin bot is admin@example.com, user bots are bot1@example.com, bot2@example.com, etc.
/// Creates accounts if they don't exist.
///
/// Available only from local bot API port.
#[utoipa::path(
    post,
    path = PATH_GET_BOTS,
    security(),
    responses(
        (status = 200, description = "Bot accounts retrieved.", body = GetBotsResult),
        (status = 500, description = "Internal server error."),
    )
)]
pub async fn post_get_bots(State(state): State<S>) -> Result<Json<GetBotsResult>, StatusCode> {
    ACCOUNT_BOT.post_get_bots.incr();
    get_or_create_bots_impl(&state).await
}

pub const PATH_REMOTE_GET_BOTS: &str = "/account_api/remote_get_bots";

/// Get admin and user bot accounts by email pattern.
/// Admin bot is admin@example.com, user bots are bot1@example.com, bot2@example.com, etc.
/// Creates accounts if they don't exist.
///
/// Available only from public and local bot API ports.
#[utoipa::path(
    post,
    path = PATH_REMOTE_GET_BOTS,
    security(),
    request_body = RemoteBotPassword,
    responses(
        (status = 200, description = "Bot accounts retrieved.", body = GetBotsResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_remote_get_bots(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Json(info): Json<RemoteBotPassword>,
) -> Result<Json<GetBotsResult>, StatusCode> {
    ACCOUNT_BOT.post_remote_get_bots.incr();

    if !state.is_remote_bot_login_enabled() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let config = state.config().remote_bot_login();

    if let Some(access_config) = config.access()
        && !is_ip_address_accepted(&state, address, access_config).await
    {
        return Err(StatusCode::NOT_FOUND);
    }

    let Some(configured_password) = config.password() else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if configured_password != info.password {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    get_or_create_bots_impl(&state).await
}

/// Get or create bot accounts based on server configuration.
/// Creates accounts that don't exist.
async fn get_or_create_bots_impl(state: &S) -> Result<Json<GetBotsResult>, StatusCode> {
    // Get existing bot accounts using data layer
    let mut result = state.read().account().get_existing_bots().await?;

    const ADMIN_EMAIL: &str = "admin@example.com";
    const BOT_EMAIL_PREFIX: &str = "bot";
    const BOT_EMAIL_SUFFIX: &str = "@example.com";

    // Get bot config to determine expected user bot count
    let bot_config = state
        .read()
        .common()
        .bot_config()
        .bot_config()
        .await?
        .unwrap_or_default();
    let expected_user_count = bot_config.user_bots as usize;

    // Create missing admin bot if needed
    if bot_config.admin_bot
        && result.admin.is_none()
        && let Some(admin) =
            create_bot_account(state, EmailAddress(ADMIN_EMAIL.to_string())).await?
    {
        result.admin = Some(admin);
    }

    // Create missing user bots if needed
    if result.users.len() < expected_user_count {
        for i in result.users.len()..expected_user_count {
            let bot_number = i + 1; // Start from bot1, not bot0
            let bot_email = EmailAddress(format!(
                "{}{}{}",
                BOT_EMAIL_PREFIX, bot_number, BOT_EMAIL_SUFFIX
            ));
            if let Some(bot) = create_bot_account(state, bot_email).await? {
                result.users.push(bot);
            }
        }
    }

    Ok(result.into())
}

/// Helper to create a new bot account
async fn create_bot_account(
    state: &S,
    email: EmailAddress,
) -> Result<Option<BotAccount>, StatusCode> {
    let new_account_id = state
        .data_all_access()
        .register_impl(SignInWithInfo::default(), None)
        .await?;

    db_write!(state, move |cmds| {
        cmds.common()
            .set_is_bot_account(new_account_id, true)
            .await?;
        cmds.account()
            .email()
            .inital_setup_account_email_change(new_account_id, email)
            .await
    })?;

    Ok(Some(BotAccount {
        aid: new_account_id.as_id(),
    }))
}

pub const PATH_REMOTE_BOT_LOGIN: &str = "/account_api/remote_bot_login";

/// Login for remote bots.
///
/// Available only from public and local bot API ports.
#[utoipa::path(
    post,
    path = PATH_REMOTE_BOT_LOGIN,
    security(),
    request_body = RemoteBotLogin,
    responses(
        (status = 200, description = "Login successful.", body = LoginResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_remote_bot_login(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Json(info): Json<RemoteBotLogin>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT_BOT.post_remote_bot_login.incr();

    if !state.is_remote_bot_login_enabled() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let config = state.config().remote_bot_login();

    if let Some(access_config) = config.access()
        && !is_ip_address_accepted(&state, address, access_config).await
    {
        return Err(StatusCode::NOT_FOUND);
    }

    let Some(configured_password) = config.password() else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if configured_password != info.password {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let internal_id = state.get_internal_id(info.aid).await?;
    let is_bot = state.read().account().is_bot_account(internal_id).await?;
    if !is_bot {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let r = login_impl(info.aid, address, &state).await?;

    if let Some(aid) = r.aid() {
        // Login successful
        let id = state.get_internal_id(aid).await?;
        db_write!(state, move |cmds| {
            cmds.common()
                .client_config()
                .client_login_session_platform(id, ClientType::Bot)
                .await
        })?;
    }

    Ok(r.into())
}

create_counters!(
    AccountBotCounters,
    ACCOUNT_BOT,
    ACCOUNT_BOT_COUNTERS_LIST,
    post_bot_login,
    post_bot_register,
    post_remote_bot_login,
    post_get_bots,
    post_remote_get_bots,
);
