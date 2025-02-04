//! Account related internal API routes

use std::{collections::HashSet, sync::LazyLock};

use axum::extract::State;
use model_account::{AccountId, RemoteBotLogin, LoginResult, SignInWithInfo};
use server_api::{app::GetConfig, db_write, S};
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

pub const PATH_LOGIN: &str = "/account_api/login";

/// Get new AccessToken for a bot account. If the account is not registered
/// as a bot account, then the request will fail.
///
/// Available only from bot API port.
#[utoipa::path(
    post,
    path = "/account_api/login",
    security(),
    request_body = AccountId,
    responses(
        (status = 200, description = "Login successful.", body = LoginResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_login(
    State(state): State<S>,
    Json(id): Json<AccountId>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT_BOT.post_login.incr();

    let internal_id = state.get_internal_id(id).await?;
    let is_bot = state.read().account().is_bot_account(internal_id).await?;
    if !is_bot {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    login_impl(id, state).await.map(|d| d.into())
}

pub const PATH_REGISTER: &str = "/account_api/register";

/// Register a new bot account. Returns new account ID which is UUID.
///
/// Available only from bot API port.
#[utoipa::path(
    post,
    path = "/account_api/register",
    security(),
    responses(
        (status = 200, description = "New profile created.", body = AccountId),
        (status = 500, description = "Internal server error."),
    )
)]
pub async fn post_register(State(state): State<S>) -> Result<Json<AccountId>, StatusCode> {
    ACCOUNT_BOT.post_register.incr();
    let new_account_id = state
        .data_all_access()
        .register_impl(SignInWithInfo::default(), None)
        .await?;

    db_write!(state, move |cmds| {
        cmds.common().set_is_bot_account(new_account_id, true)
    })?;

    // TODO(microservice): The is_bot_account is currently not synced
    // to other servers.

    Ok(new_account_id.as_id().into())
}

pub struct RemoteBotLoginState {
    blocked: HashSet<AccountId>,
}

static REMOTE_BOT_LOGIN_STATE: LazyLock<Mutex<RemoteBotLoginState>> = std::sync::LazyLock::new(
    || Mutex::new(
        RemoteBotLoginState { blocked: HashSet::new() }
    )
);

pub const PATH_REMOTE_BOT_LOGIN: &str = "/account_api/remote_bot_login";

/// Login for remote bots which are listed in server config file.
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
    Json(info): Json<RemoteBotLogin>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT_BOT.post_remote_bot_login.incr();

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

    let bots = state.config().remote_bots();
    let Some(configured_bot) = bots.iter().find(|v| v.account_id == info.aid) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    if configured_bot.password != info.password {
        info!("Remote bot login failed. Wrong password for account {}", info.aid);
        bot_login_state.blocked.insert(info.aid);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    login_impl(info.aid, state).await.map(|d| d.into())
}

create_counters!(
    AccountBotCounters,
    ACCOUNT_BOT,
    ACCOUNT_BOT_COUNTERS_LIST,
    post_login,
    post_register,
    post_remote_bot_login,
);
