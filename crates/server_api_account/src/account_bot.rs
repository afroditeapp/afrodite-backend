//! Account related internal API routes

use axum::extract::State;
use model_account::{AccountId, LoginResult, SignInWithInfo};
use server_api::{db_write, S};
use server_data::write::GetWriteCommandsCommon;
use server_data_account::read::GetReadCommandsAccount;
use simple_backend::create_counters;

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

create_counters!(
    AccountBotCounters,
    ACCOUNT_BOT,
    ACCOUNT_BOT_COUNTERS_LIST,
    post_login,
    post_register,
);
