use std::{collections::HashSet, net::SocketAddr, sync::LazyLock};

use axum::extract::{ConnectInfo, State};
use model_account::{AccountId, LoginResult, RemoteBotLogin, SignInWithInfo};
use server_api::{
    S,
    app::{GetConfig, ReadDynamicConfig},
    common::is_ip_address_accepted,
    db_write,
};
use server_data::write::GetWriteCommandsCommon;
use server_data_account::read::GetReadCommandsAccount;
use simple_backend::create_counters;
use tokio::sync::Mutex;
use tracing::info;

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

    login_impl(id, address, &state).await.map(|d| d.into())
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

pub struct RemoteBotLoginState {
    blocked: HashSet<AccountId>,
}

static REMOTE_BOT_LOGIN_STATE: LazyLock<Mutex<RemoteBotLoginState>> =
    std::sync::LazyLock::new(|| {
        Mutex::new(RemoteBotLoginState {
            blocked: HashSet::new(),
        })
    });

pub const PATH_REMOTE_BOT_LOGIN: &str = "/account_api/remote_bot_login";

/// Login for remote bots which are listed in server config file.
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

    let bots = state.config().remote_bots();
    let Some(configured_bot) = bots.iter().find(|v| v.account_id() == info.aid) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if !is_ip_address_accepted(&state, address, configured_bot.access()).await {
        return Err(StatusCode::NOT_FOUND);
    }

    let internal_id = state.get_internal_id(info.aid).await?;
    let is_bot = state.read().account().is_bot_account(internal_id).await?;
    if !is_bot {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let mut bot_login_state = REMOTE_BOT_LOGIN_STATE.lock().await;
    if bot_login_state.blocked.contains(&info.aid) {
        info!("Remote bot login is blocked for account {}", info.aid);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if configured_bot.password() != info.password {
        info!(
            "Remote bot login failed. Wrong password for account {}",
            info.aid
        );
        bot_login_state.blocked.insert(info.aid);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    login_impl(info.aid, address, &state)
        .await
        .map(|d| d.into())
}

create_counters!(
    AccountBotCounters,
    ACCOUNT_BOT,
    ACCOUNT_BOT_COUNTERS_LIST,
    post_bot_login,
    post_bot_register,
    post_remote_bot_login,
);
