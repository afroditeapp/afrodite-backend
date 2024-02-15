use axum::{extract::State, Extension, Router};
use model::{Account, AccountIdInternal};
use simple_backend::create_counters;

use crate::{
    api::utils::{Json, StatusCode},
    app::{GetAccessTokens, ReadData},
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
        .common()
        .account(api_caller_account_id)
        .await?;
    Ok(account.into())
}

pub fn state_router(s: crate::app::S) -> Router {
    use axum::routing::get;

    use crate::app::S;

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
