use std::time::Duration;

use axum::extract::State;
use model_account::{AccessibleAccount, AccountId, DemoModeLoginToAccount, LoginResult, SignInWithInfo};
use model_server_state::{
    DemoModeConfirmLogin, DemoModeConfirmLoginResult, DemoModeLoginResult,
    DemoModePassword, DemoModeToken,
};
use server_api::{
    app::{GetConfig, ReadData}, create_open_api_router, db_write_multiple, S
};
use server_data_account::{
    demo::{AccessibleAccountsInfoUtils, DemoModeUtils},
    write::GetWriteCommandsAccount,
};
use simple_backend::create_counters;

use super::login_impl;
use crate::{
    app::WriteData,
    utils::{Json, StatusCode},
};

// TODO(prod): Use one route for login and change wording to user ID and
//             password? Also info about locked account only if password
//             is correct?

const PATH_POST_DEMO_MODE_LOGIN: &str = "/account_api/demo_mode_login";

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
pub async fn post_demo_mode_login(
    State(state): State<S>,
    Json(password): Json<DemoModePassword>,
) -> Result<Json<DemoModeLoginResult>, StatusCode> {
    ACCOUNT.post_demo_mode_login.incr();
    // TODO(prod): Increase to 5 seconds
    tokio::time::sleep(Duration::from_secs(1)).await;
    let result = state.demo_mode().stage0_login(password).await?;
    Ok(result.into())
}

const PATH_POST_DEMO_MODE_CONFIRM_LOGIN: &str = "/account_api/demo_mode_confirm_login";

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
pub async fn post_demo_mode_confirm_login(
    State(state): State<S>,
    Json(info): Json<DemoModeConfirmLogin>,
) -> Result<Json<DemoModeConfirmLoginResult>, StatusCode> {
    ACCOUNT.post_demo_mode_confirm_login.incr();
    let result = state
        .demo_mode()
        .stage1_login(info.password, info.token)
        .await?;
    Ok(result.into())
}

const PATH_POST_DEMO_MODE_ACCESSIBLE_ACCOUNTS: &str = "/account_api/demo_mode_accessible_accounts";

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
pub async fn post_demo_mode_accessible_accounts(
    State(state): State<S>,
    Json(token): Json<DemoModeToken>,
) -> Result<Json<Vec<AccessibleAccount>>, StatusCode> {
    ACCOUNT.post_demo_mode_accessible_accounts.incr();

    let info = state
        .demo_mode()
        .accessible_accounts_if_token_valid(&token)
        .await?;
    let accounts = info.into_accounts(state.read()).await?;
    let result = DemoModeUtils::with_extra_info(accounts, state.config(), state.read()).await?;

    Ok(result.into())
}

const PATH_POST_DEMO_MODE_REGISTER_ACCOUNT: &str = "/account_api/demo_mode_register_account";

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
pub async fn post_demo_mode_register_account(
    State(state): State<S>,
    Json(token): Json<DemoModeToken>,
) -> Result<Json<AccountId>, StatusCode> {
    ACCOUNT.post_demo_mode_register_account.incr();

    let demo_mode_id = state.demo_mode().demo_mode_token_exists(&token).await?;

    let id = state
        .data_all_access()
        .register_impl(SignInWithInfo::default(), None)
        .await?;

    db_write_multiple!(state, move |cmds| cmds
        .account()
        .insert_demo_mode_related_account_ids(demo_mode_id, id.as_id())
        .await)?;

    Ok(id.as_id().into())
}

const PATH_POST_DEMO_MODE_LOGIN_TO_ACCOUNT: &str = "/account_api/demo_mode_login_to_account";

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
pub async fn post_demo_mode_login_to_account(
    State(state): State<S>,
    Json(info): Json<DemoModeLoginToAccount>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT.post_demo_mode_login_to_account.incr();

    if let Some(min_version) = state.config().min_client_version() {
        if !min_version.received_version_is_accepted(info.client_info.client_version) {
            return Ok(LoginResult::error_unsupported_client().into());
        }
    }

    let accessible_accounts = state
        .demo_mode()
        .accessible_accounts_if_token_valid(&info.token)
        .await?;
    accessible_accounts.contains(info.aid, state.read()).await?;

    let result = login_impl(info.aid, state).await?;

    Ok(result.into())
}

const PATH_POST_DEMO_MODE_LOGOUT: &str = "/account_api/demo_mode_logout";

#[utoipa::path(
    post,
    path = PATH_POST_DEMO_MODE_LOGOUT,
    request_body = DemoModeToken,
    responses(
        (status = 200, description = "Successfull."),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_demo_mode_logout(
    State(state): State<S>,
    Json(token): Json<DemoModeToken>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_demo_mode_logout.incr();
    state.demo_mode().demo_mode_logout(&token).await?;
    Ok(())
}

create_open_api_router!(
        fn router_demo_mode,
        post_demo_mode_accessible_accounts,
        post_demo_mode_login,
        post_demo_mode_confirm_login,
        post_demo_mode_register_account,
        post_demo_mode_login_to_account,
        post_demo_mode_logout,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_DEMO_MODE_COUNTERS_LIST,
    post_demo_mode_accessible_accounts,
    post_demo_mode_login,
    post_demo_mode_confirm_login,
    post_demo_mode_register_account,
    post_demo_mode_login_to_account,
    post_demo_mode_logout,
);
