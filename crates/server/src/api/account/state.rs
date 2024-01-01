
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



pub const PATH_ACCOUNT_STATE: &str = "/account_api/state";

/// Get current account state.
#[utoipa::path(
    get,
    path = "/account_api/state",
    responses(
        (status = 200, description = "Request successfull.", body = Account),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_account_state<S: GetAccessTokens + ReadData>(
    State(state): State<S>,
    Extension(api_caller_account_id): Extension<AccountIdInternal>,
) -> Result<Json<Account>, StatusCode> {
    ACCOUNT.get_account_state.incr();
    let account = state
        .read()
        .account()
        .account(api_caller_account_id)
        .await?;
    Ok(account.into())
}

pub fn state_router(s: crate::app::S) -> Router {
    use crate::app::S;
    use axum::routing::{get, post, put};

    Router::new()
        .route(PATH_ACCOUNT_STATE, get(get_account_state::<S>))
        .with_state(s)
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_STATE_COUNTERS_LIST,
    get_account_state,
);
