use axum::{extract::State, Extension, Router};
use model::{Account, AccountIdInternal, LatestBirthdate};
use server_data::read::GetReadCommandsCommon;
use simple_backend::create_counters;

use crate::{
    app::{GetAccessTokens, ReadData, StateBase},
    utils::{Json, StatusCode},
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
    let account = state.read().common().account(api_caller_account_id).await?;
    Ok(account.into())
}

pub const PATH_LATEST_BIRTHDATE: &str = "/account_api/latest_birthdate";

#[utoipa::path(
    get,
    path = PATH_LATEST_BIRTHDATE,
    responses(
        (status = 200, description = "Request successfull.", body = LatestBirthdate),
        (status = 401, description = "Unauthorized."),
        (status = 500, description = "Internal server error."),
    ),
    security(("access_token" = [])),
)]
pub async fn get_latest_birthdate<S: GetAccessTokens + ReadData>(
    State(state): State<S>,
    Extension(id): Extension<AccountIdInternal>,
) -> Result<Json<LatestBirthdate>, StatusCode> {
    ACCOUNT.get_latest_birthdate.incr();
    let birthdate = state.read().common().latest_birthdate(id).await?;
    let birthdate = LatestBirthdate {
        birthdate
    };
    Ok(birthdate.into())
}

pub fn state_router<S: StateBase + GetAccessTokens + ReadData>(s: S) -> Router {
    use axum::routing::get;

    Router::new()
        .route(PATH_ACCOUNT_STATE, get(get_account_state::<S>))
        .route(PATH_LATEST_BIRTHDATE, get(get_latest_birthdate::<S>))
        .with_state(s)
}

create_counters!(
    AccountCounters,
    ACCOUNT,
    ACCOUNT_STATE_COUNTERS_LIST,
    get_account_state,
    get_latest_birthdate,
);
