use std::time::Duration;

use axum::extract::State;
use model_account::{
    AccessibleAccount, AccountId, DemoModeConfirmLogin, DemoModeConfirmLoginResult, DemoModeId,
    DemoModeLoginResult, DemoModeLoginToAccount, DemoModePassword, DemoModeToken, LoginResult,
    SignInWithInfo,
};
use obfuscate_api_macro::obfuscate_api;
use server_api::S;
use server_api::{create_open_api_router, db_write};
use server_data_account::write::GetWriteCommandsAccount;
use simple_backend::create_counters;
use utoipa_axum::router::OpenApiRouter;

use super::login_impl;
use crate::{
    app::{DemoModeManagerProvider, WriteData},
    utils::{Json, StatusCode},
};

// TODO(prod): Use one route for login and change wording to user ID and
//             password? Also info about locked account only if password
//             is correct?

#[obfuscate_api]
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
    let result = state.stage0_login(password).await?;
    Ok(result.into())
}

#[obfuscate_api]
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
    let result = state.stage1_login(info.password, info.token).await?;
    Ok(result.into())
}

#[obfuscate_api]
const PATH_POST_DEMO_MODE_ACCESSIBLE_ACCOUNTS: &str =
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
pub async fn post_demo_mode_accessible_accounts(
    State(state): State<S>,
    Json(token): Json<DemoModeToken>,
) -> Result<Json<Vec<AccessibleAccount>>, StatusCode> {
    ACCOUNT.post_demo_mode_accessible_accounts.incr();
    let result = state
        .accessible_accounts_if_token_valid(&token)
        .await?;
    Ok(result.into())
}

#[obfuscate_api]
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

    let demo_mode_id = state.demo_mode_token_exists(&token).await?;

    let id = state.register_impl(SignInWithInfo::default(), None).await?;

    db_write!(state, move |cmds| cmds
        .account()
        .insert_demo_mode_related_account_ids(demo_mode_id, id.as_id()))?;

    Ok(id.as_id().into())
}

#[obfuscate_api]
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

    let _demo_mode_id: DemoModeId = state.demo_mode_token_exists(&info.token).await?;

    let result = login_impl(info.aid, state).await?;

    Ok(result.into())
}

#[obfuscate_api]
const PATH_POST_DEMO_MODE_LOGOUT: &str =
    "/account_api/demo_mode_logout";

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
    state.demo_mode_logout(&token).await?;
    Ok(())
}

pub fn demo_mode_router(
    s: S,
) -> OpenApiRouter {
    create_open_api_router!(
        s,
        post_demo_mode_accessible_accounts,
        post_demo_mode_login,
        post_demo_mode_confirm_login,
        post_demo_mode_register_account,
        post_demo_mode_login_to_account,
        post_demo_mode_logout,
    )
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
    post_demo_mode_logout,
);
