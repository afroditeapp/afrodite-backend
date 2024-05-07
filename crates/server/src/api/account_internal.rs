//! Account related internal API routes

use axum::extract::State;
use model::{
    AccessToken, AccountId, AuthPair, GoogleAccountId, LoginResult, RefreshToken, SignInWithInfo,
    SignInWithLoginInfo,
};
use simple_backend::{app::SignInWith, create_counters};

use crate::{
    api::{
        db_write,
        utils::{Json, StatusCode},
    },
    app::{GetAccessTokens, GetAccounts, GetConfig, ReadData, WriteData},
};

use super::account::{login_impl, register_impl};

pub const PATH_LOGIN: &str = "/account_api/login";

/// Get new AccessToken.
///
/// Available only if server is running in debug mode and
/// bot_login is enabled from config file.
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
pub async fn post_login<S: WriteData + GetAccounts>(
    State(state): State<S>,
    Json(id): Json<AccountId>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT_INTERNAL.post_login.incr();
    login_impl(id, state).await.map(|d| d.into())
}

pub const PATH_REGISTER: &str = "/account_api/register";

/// Register new account. Returns new account ID which is UUID.
///
/// Available only if server is running in debug mode and
/// bot_login is enabled from config file.
#[utoipa::path(
    post,
    path = "/account_api/register",
    security(),
    responses(
        (status = 200, description = "New profile created.", body = AccountId),
        (status = 500, description = "Internal server error."),
    )
)]
pub async fn post_register<S: WriteData + GetConfig>(
    State(state): State<S>,
) -> Result<Json<AccountId>, StatusCode> {
    ACCOUNT_INTERNAL.post_register.incr();
    register_impl(&state, SignInWithInfo::default())
        .await
        .map(|id| id.into())
}

create_counters!(
    AccountInternalCounters,
    ACCOUNT_INTERNAL,
    ACCOUNT_INTERNAL_COUNTERS_LIST,
    post_login,
    post_register,
);
