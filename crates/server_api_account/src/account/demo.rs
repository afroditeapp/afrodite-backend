use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use axum::extract::{ConnectInfo, State};
use model_account::{
    AccessibleAccount, AccountId, DemoAccountLoginToAccount, LoginResult, SignInWithInfo,
};
use model_server_state::{DemoAccountLoginCredentials, DemoAccountLoginResult, DemoAccountToken};
use server_api::{
    S,
    app::{GetAccounts, GetConfig, ReadData},
    create_open_api_router, db_write,
};
use server_data::write::GetWriteCommandsCommon;
use server_data_account::{
    demo::{AccessibleAccountsInfoUtils, DemoAccountUtils},
    write::GetWriteCommandsAccount,
};
use simple_backend::create_counters;

use super::login_impl;
use crate::{
    app::WriteData,
    utils::{Json, StatusCode},
};

const PATH_POST_DEMO_ACCOUNT_LOGIN: &str = "/account_api/demo_account_login";

/// Access demo account, which allows accessing all or specific accounts
/// depending on the server configuration.
///
/// This API route has 1 second wait time to make password guessing harder.
/// Account will be locked if the password is guessed. Server process restart
/// will reset the lock.
#[utoipa::path(
    post,
    path = PATH_POST_DEMO_ACCOUNT_LOGIN,
    request_body = DemoAccountLoginCredentials,
    responses(
        (status = 200, description = "Successfull.", body = DemoAccountLoginResult),
    ),
    security(),
)]
pub async fn post_demo_account_login(
    State(state): State<S>,
    Json(credentials): Json<DemoAccountLoginCredentials>,
) -> Result<Json<DemoAccountLoginResult>, StatusCode> {
    ACCOUNT.post_demo_account_login.incr();

    let wait_until = Instant::now() + Duration::from_secs(1);
    let result = state.demo().login(credentials).await;
    tokio::time::sleep_until(wait_until.into()).await;

    Ok(result.into())
}

const PATH_POST_DEMO_ACCOUNT_ACCESSIBLE_ACCOUNTS: &str =
    "/account_api/demo_account_accessible_accounts";

/// Get demo account's available accounts.
///
/// This path is using HTTP POST because there is JSON in the request body.
#[utoipa::path(
    post,
    path = PATH_POST_DEMO_ACCOUNT_ACCESSIBLE_ACCOUNTS,
    request_body = DemoAccountToken,
    responses(
        (status = 200, description = "Successfull.", body = Vec<AccessibleAccount>),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_demo_account_accessible_accounts(
    State(state): State<S>,
    Json(token): Json<DemoAccountToken>,
) -> Result<Json<Vec<AccessibleAccount>>, StatusCode> {
    ACCOUNT.post_demo_account_accessible_accounts.incr();

    let Some(id) = state.demo().valid_demo_account_token_exists(&token).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let info = state.demo().accessible_accounts(id).await?;
    let accounts = info.into_accounts(state.read()).await?;
    let result = DemoAccountUtils::with_extra_info(accounts, state.read()).await?;

    Ok(result.into())
}

const PATH_POST_DEMO_ACCOUNT_REGISTER_ACCOUNT: &str = "/account_api/demo_account_register_account";

#[utoipa::path(
    post,
    path = PATH_POST_DEMO_ACCOUNT_REGISTER_ACCOUNT,
    request_body = DemoAccountToken,
    responses(
        (status = 200, description = "Successful.", body = AccountId),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_demo_account_register_account(
    State(state): State<S>,
    Json(token): Json<DemoAccountToken>,
) -> Result<Json<AccountId>, StatusCode> {
    ACCOUNT.post_demo_account_register_account.incr();

    let Some(demo_account_id) = state.demo().valid_demo_account_token_exists(&token).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let id = state
        .data_all_access()
        .register_impl(SignInWithInfo::default(), None)
        .await?;

    db_write!(state, move |cmds| cmds
        .account()
        .add_to_demo_account_owned_accounts(demo_account_id, id)
        .await)?;

    Ok(id.as_id().into())
}

const PATH_POST_DEMO_ACCOUNT_LOGIN_TO_ACCOUNT: &str = "/account_api/demo_account_login_to_account";

#[utoipa::path(
    post,
    path = PATH_POST_DEMO_ACCOUNT_LOGIN_TO_ACCOUNT,
    request_body = DemoAccountLoginToAccount,
    responses(
        (status = 200, description = "Successful.", body = LoginResult),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_demo_account_login_to_account(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Json(info): Json<DemoAccountLoginToAccount>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT.post_demo_account_login_to_account.incr();

    let Some(id) = state
        .demo()
        .valid_demo_account_token_exists(&info.token)
        .await
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Some(min_version) = state.config().min_client_version() {
        if !min_version.received_version_is_accepted(info.client_info.client_version) {
            return Ok(LoginResult::error_unsupported_client().into());
        }
    }

    let accessible_accounts = state.demo().accessible_accounts(id).await?;
    accessible_accounts.contains(info.aid, state.read()).await?;

    let r = login_impl(info.aid, address, &state).await?;

    if let Some(aid) = r.aid {
        // Login successful
        let id = state.get_internal_id(aid).await?;
        db_write!(state, move |cmds| {
            cmds.common()
                .client_config()
                .client_login_session_platform(id, info.client_info.client_type)
                .await
        })?;
    }

    Ok(r.into())
}

const PATH_POST_DEMO_ACCOUNT_LOGOUT: &str = "/account_api/demo_account_logout";

#[utoipa::path(
    post,
    path = PATH_POST_DEMO_ACCOUNT_LOGOUT,
    request_body = DemoAccountToken,
    responses(
        (status = 200, description = "Successfull."),
    ),
    security(),
)]
pub async fn post_demo_account_logout(
    State(state): State<S>,
    Json(token): Json<DemoAccountToken>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_demo_account_logout.incr();
    state.demo().demo_account_logout(&token).await;
    Ok(())
}

create_open_api_router!(
        fn router_demo,
        post_demo_account_accessible_accounts,
        post_demo_account_login,
        post_demo_account_register_account,
        post_demo_account_login_to_account,
        post_demo_account_logout,
);

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_DEMO_COUNTERS_LIST,
    post_demo_account_accessible_accounts,
    post_demo_account_login,
    post_demo_account_register_account,
    post_demo_account_login_to_account,
    post_demo_account_logout,
);
