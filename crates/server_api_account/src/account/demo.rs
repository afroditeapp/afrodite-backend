use std::{net::SocketAddr, time::Duration};

use axum::extract::{ConnectInfo, State};
use model_account::{
    AccessibleAccount, AccountId, DemoModeLoginToAccount, LoginResult, SignInWithInfo,
};
use model_server_state::{DemoModeLoginCredentials, DemoModeLoginResult, DemoModeToken};
use server_api::{
    S,
    app::{GetAccounts, GetConfig, ReadData},
    create_open_api_router, db_write,
};
use server_data::write::GetWriteCommandsCommon;
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

const PATH_POST_DEMO_MODE_LOGIN: &str = "/account_api/demo_mode_login";

/// Access demo mode, which allows accessing all or specific accounts
/// depending on the server configuration.
#[utoipa::path(
    post,
    path = PATH_POST_DEMO_MODE_LOGIN,
    request_body = DemoModeLoginCredentials,
    responses(
        (status = 200, description = "Successfull.", body = DemoModeLoginResult),
    ),
    security(),
)]
pub async fn post_demo_mode_login(
    State(state): State<S>,
    Json(credentials): Json<DemoModeLoginCredentials>,
) -> Result<Json<DemoModeLoginResult>, StatusCode> {
    ACCOUNT.post_demo_mode_login.incr();
    // TODO(prod): Increase to 5 seconds
    tokio::time::sleep(Duration::from_secs(1)).await;
    let result = state.demo_mode().login(credentials).await;
    Ok(result.into())
}

const PATH_POST_DEMO_MODE_ACCESSIBLE_ACCOUNTS: &str = "/account_api/demo_mode_accessible_accounts";

/// Get demo account's available accounts.
///
/// This path is using HTTP POST because there is JSON in the request body.
#[utoipa::path(
    post,
    path = PATH_POST_DEMO_MODE_ACCESSIBLE_ACCOUNTS,
    request_body = DemoModeToken,
    responses(
        (status = 200, description = "Successfull.", body = Vec<AccessibleAccount>),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_demo_mode_accessible_accounts(
    State(state): State<S>,
    Json(token): Json<DemoModeToken>,
) -> Result<Json<Vec<AccessibleAccount>>, StatusCode> {
    ACCOUNT.post_demo_mode_accessible_accounts.incr();

    let Some(id) = state.demo_mode().valid_demo_mode_token_exists(&token).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let info = state.demo_mode().accessible_accounts(id).await?;
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
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_demo_mode_register_account(
    State(state): State<S>,
    Json(token): Json<DemoModeToken>,
) -> Result<Json<AccountId>, StatusCode> {
    ACCOUNT.post_demo_mode_register_account.incr();

    let Some(demo_mode_id) = state.demo_mode().valid_demo_mode_token_exists(&token).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let id = state
        .data_all_access()
        .register_impl(SignInWithInfo::default(), None)
        .await?;

    db_write!(state, move |cmds| cmds
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
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(),
)]
pub async fn post_demo_mode_login_to_account(
    State(state): State<S>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Json(info): Json<DemoModeLoginToAccount>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT.post_demo_mode_login_to_account.incr();

    let Some(id) = state
        .demo_mode()
        .valid_demo_mode_token_exists(&info.token)
        .await
    else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if let Some(min_version) = state.config().min_client_version() {
        if !min_version.received_version_is_accepted(info.client_info.client_version) {
            return Ok(LoginResult::error_unsupported_client().into());
        }
    }

    let accessible_accounts = state.demo_mode().accessible_accounts(id).await?;
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

const PATH_POST_DEMO_MODE_LOGOUT: &str = "/account_api/demo_mode_logout";

#[utoipa::path(
    post,
    path = PATH_POST_DEMO_MODE_LOGOUT,
    request_body = DemoModeToken,
    responses(
        (status = 200, description = "Successfull."),
    ),
    security(),
)]
pub async fn post_demo_mode_logout(
    State(state): State<S>,
    Json(token): Json<DemoModeToken>,
) -> Result<(), StatusCode> {
    ACCOUNT.post_demo_mode_logout.incr();
    state.demo_mode().demo_mode_logout(&token).await;
    Ok(())
}

create_open_api_router!(
        fn router_demo_mode,
        post_demo_mode_accessible_accounts,
        post_demo_mode_login,
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
    post_demo_mode_register_account,
    post_demo_mode_login_to_account,
    post_demo_mode_logout,
);
