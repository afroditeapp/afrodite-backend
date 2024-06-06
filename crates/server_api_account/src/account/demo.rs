use std::time::Duration;

use axum::{extract::State, Router};
use model::{
    AccessibleAccount, AccountId, DemoModeConfirmLogin, DemoModeConfirmLoginResult, DemoModeId,
    DemoModeLoginResult, DemoModeLoginToAccount, DemoModePassword, DemoModeToken, LoginResult,
    SignInWithInfo,
};
use server_api::{app::{RegisteringCmd, ResetPushNotificationTokens}, db_write};
use server_data_account::write::GetWriteCommandsAccount;
use simple_backend::create_counters;

use super::login_impl;
use crate::{
    app::{DemoModeManagerProvider, GetAccounts, GetConfig, ReadData, StateBase, WriteData},
    utils::{Json, StatusCode},
};

// TODO(prod): Logout route for demo account?
// TODO(prod): Use one route for login and change wording to user ID and
//             password? Also info about locked account only if password
//             is correct?

pub const PATH_POST_DEMO_MODE_LOGIN: &str = "/account_api/demo_mode_login";

/// Access demo mode, which allows accessing all or specific accounts
/// depending on the server configuration.
#[utoipa::path(
    post,
    path = PATH_POST_DEMO_MODE_LOGIN,
    request_body = DemoModePassword,
    responses(
        (status = 200, description = "Successfull.", body = DemoModeLoginResult),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_demo_mode_login<S: DemoModeManagerProvider>(
    State(state): State<S>,
    Json(password): Json<DemoModePassword>,
) -> Result<Json<DemoModeLoginResult>, StatusCode> {
    ACCOUNT.post_demo_mode_login.incr();
    // TODO(prod): Increase to 5 seconds
    tokio::time::sleep(Duration::from_secs(1)).await;
    let result = state.stage0_login(password).await?;
    Ok(result.into())
}

pub const PATH_POST_DEMO_MODE_CONFIRM_LOGIN: &str = "/account_api/demo_mode_confirm_login";

#[utoipa::path(
    post,
    path = PATH_POST_DEMO_MODE_CONFIRM_LOGIN,
    request_body = DemoModeConfirmLogin,
    responses(
        (status = 200, description = "Successfull.", body = DemoModeConfirmLoginResult),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_demo_mode_confirm_login<S: DemoModeManagerProvider>(
    State(state): State<S>,
    Json(info): Json<DemoModeConfirmLogin>,
) -> Result<Json<DemoModeConfirmLoginResult>, StatusCode> {
    ACCOUNT.post_demo_mode_confirm_login.incr();
    let result = state.stage1_login(info.password, info.token).await?;
    Ok(result.into())
}

pub const PATH_POST_DEMO_MODE_ACCESSIBLE_ACCOUNTS: &str =
    "/account_api/demo_mode_accessible_accounts";

// TODO: Return Unauthorized instead of internal server error on routes which
// require DemoModeToken?

/// Get demo account's available accounts.
///
/// This path is using HTTP POST because there is JSON in the request body.
#[utoipa::path(
    post,
    path = PATH_POST_DEMO_MODE_ACCESSIBLE_ACCOUNTS,
    request_body = DemoModeToken,
    responses(
        (status = 200, description = "Successfull.", body = Vec<AccessibleAccount>),
        (status = 500, description = "Unauthorized or internal server error."),
    ),
    security(),
)]
pub async fn post_demo_mode_accessible_accounts<
    S: DemoModeManagerProvider + ReadData + GetAccounts + GetConfig,
>(
    State(state): State<S>,
    Json(token): Json<DemoModeToken>,
) -> Result<Json<Vec<AccessibleAccount>>, StatusCode> {
    ACCOUNT.post_demo_mode_accessible_accounts.incr();
    let result = state
        .accessible_accounts_if_token_valid(&state, &token)
        .await?;
    Ok(result.into())
}

pub const PATH_POST_DEMO_MODE_REGISTER_ACCOUNT: &str = "/account_api/demo_mode_register_account";

#[utoipa::path(
    post,
    path = PATH_POST_DEMO_MODE_REGISTER_ACCOUNT,
    request_body = DemoModeToken,
    responses(
        (status = 200, description = "Successful.", body = AccountId),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_demo_mode_register_account<
    S: DemoModeManagerProvider + WriteData + GetConfig + RegisteringCmd,
>(
    State(state): State<S>,
    Json(token): Json<DemoModeToken>,
) -> Result<Json<AccountId>, StatusCode> {
    ACCOUNT.post_demo_mode_register_account.incr();

    let demo_mode_id = state.demo_mode_token_exists(&token).await?;

    let id = state.register_impl(SignInWithInfo::default(), None).await?;

    db_write!(state, move |cmds| cmds
        .account()
        .insert_demo_mode_related_account_ids(demo_mode_id, id.as_id()))?;

    Ok(id.as_id().into())
}

pub const PATH_POST_DEMO_MODE_LOGIN_TO_ACCOUNT: &str = "/account_api/demo_mode_login_to_account";

#[utoipa::path(
    post,
    path = PATH_POST_DEMO_MODE_LOGIN_TO_ACCOUNT,
    request_body = DemoModeLoginToAccount,
    responses(
        (status = 200, description = "Successful.", body = LoginResult),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_demo_mode_login_to_account<
    S: DemoModeManagerProvider + ReadData + WriteData + GetAccounts + ResetPushNotificationTokens,
>(
    State(state): State<S>,
    Json(info): Json<DemoModeLoginToAccount>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT.post_demo_mode_login_to_account.incr();

    let _demo_mode_id: DemoModeId = state.demo_mode_token_exists(&info.token).await?;

    let result = login_impl(info.account_id, state).await?;

    Ok(result.into())
}

pub fn demo_mode_router<
    S: StateBase
        + DemoModeManagerProvider
        + ReadData
        + WriteData
        + GetAccounts
        + GetConfig
        + RegisteringCmd
        + ResetPushNotificationTokens,
>(
    s: S,
) -> Router {
    use axum::routing::post;

    Router::new()
        .route(
            PATH_POST_DEMO_MODE_ACCESSIBLE_ACCOUNTS,
            post(post_demo_mode_accessible_accounts::<S>),
        )
        .route(PATH_POST_DEMO_MODE_LOGIN, post(post_demo_mode_login::<S>))
        .route(
            PATH_POST_DEMO_MODE_CONFIRM_LOGIN,
            post(post_demo_mode_confirm_login::<S>),
        )
        .route(
            PATH_POST_DEMO_MODE_REGISTER_ACCOUNT,
            post(post_demo_mode_register_account::<S>),
        )
        .route(
            PATH_POST_DEMO_MODE_LOGIN_TO_ACCOUNT,
            post(post_demo_mode_login_to_account::<S>),
        )
        .with_state(s)
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_DEMO_MODE_COUNTERS_LIST,
    post_demo_mode_accessible_accounts,
    post_demo_mode_login,
    post_demo_mode_confirm_login,
    post_demo_mode_register_account,
    post_demo_mode_login_to_account,
);
