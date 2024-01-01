
use axum::{Extension, extract::State, Router};
use model::{
    AccessToken, Account, AccountData, AccountId, AccountIdInternal, AccountSetup, AccountState,
    AuthPair, BooleanSetting, DeleteStatus, EventToClientInternal, GoogleAccountId, LoginResult,
    RefreshToken, SignInWithInfo, SignInWithLoginInfo,
};
use simple_backend::{app::SignInWith, create_counters};
use tracing::error;

use crate::api::{
    db_write,
    utils::{Json, StatusCode},
};
use crate::{
    app::{
        EventManagerProvider, GetAccessTokens, GetAccounts, GetConfig, GetInternalApi, ReadData,
        WriteData,
    },
};



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
pub async fn post_login<S: GetAccessTokens + WriteData + GetAccounts>(
    State(state): State<S>,
    Json(id): Json<AccountId>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT.post_login.incr();
    login_impl(id, state).await.map(|d| d.into())
}

async fn login_impl<S: GetAccessTokens + WriteData + GetAccounts>(
    id: AccountId,
    state: S,
) -> Result<LoginResult, StatusCode> {
    let id = state.accounts().get_internal_id(id).await?;

    let access = AccessToken::generate_new();
    let refresh = RefreshToken::generate_new();
    let account = AuthPair { access, refresh };
    let account_clone = account.clone();

    db_write!(state, move |cmds| cmds.common().set_new_auth_pair(
        id,
        account_clone,
        None
    ))?;

    // TODO: microservice support

    let result = LoginResult {
        account,
        profile: None,
        media: None,
    };
    Ok(result.into())
}

pub const PATH_SIGN_IN_WITH_LOGIN: &str = "/account_api/sign_in_with_login";

/// Start new session with sign in with Apple or Google. Creates new account if
/// it does not exists.
#[utoipa::path(
    post,
    path = "/account_api/sign_in_with_login",
    security(),
    request_body = SignInWithLoginInfo,
    responses(
        (status = 200, description = "Login or account creation successful.", body = LoginResult),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn post_sign_in_with_login<
    S: GetAccessTokens + WriteData + ReadData + GetAccounts + SignInWith + GetConfig,
>(
    State(state): State<S>,
    Json(tokens): Json<SignInWithLoginInfo>,
) -> Result<Json<LoginResult>, StatusCode> {
    ACCOUNT.post_sign_in_with_login.incr();
    if let Some(google) = tokens.google_token {
        let info = state
            .sign_in_with_manager()
            .validate_google_token(google)
            .await?;
        let google_id = GoogleAccountId(info.id);
        let already_existing_account = state
            .read()
            .account()
            .google_account_id_to_account_id(google_id.clone())
            .await?;

        if let Some(already_existing_account) = already_existing_account {
            login_impl(already_existing_account.as_id(), state)
                .await
                .map(|d| d.into())
        } else {
            let id = super::register_impl(
                &state,
                SignInWithInfo {
                    google_account_id: Some(google_id),
                },
            )
            .await?;
            login_impl(id, state).await.map(|d| d.into())
        }
    } else if let Some(apple) = tokens.apple_token {
        let _info = state
            .sign_in_with_manager()
            .validate_apple_token(apple)
            .await?;

        // if validate_sign_in_with_apple_token(apple).await.unwrap() {
        //     let key = AccessToken::generate_new();
        //     Ok(key.into())
        // } else {
        //     Err(StatusCode::INTERNAL_SERVER_ERROR)
        // }
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_LOGIN_COUNTERS_LIST,
    post_login,
    post_sign_in_with_login,
);
