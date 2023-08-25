//! Handlers for internal from Server to Server state transfers and messages

use axum::extract::Path;
use hyper::StatusCode;
use model::{Account, AccountId, AccessToken};
use tracing::error;

use crate::api::{utils::Json, GetAccessTokens, GetUsers, ReadData};

pub const PATH_INTERNAL_CHECK_ACCESS_TOKEN: &str = "/internal/check_access_token";

#[utoipa::path(
    get,
    path = "/internal/check_access_token",
    request_body(content = AccessToken),
    responses(
        (status = 200, description = "Check API key", body = AccountId),
        (status = 404, description = "API key was invalid"),
    ),
    security(),
)]
pub async fn check_api_key<S: GetAccessTokens>(
    Json(api_key): Json<AccessToken>,
    state: S,
) -> Result<Json<AccountId>, StatusCode> {
    state
        .api_keys()
        .access_token_exists(&api_key)
        .await
        .ok_or(StatusCode::NOT_FOUND)
        .map(|id| id.as_light().into())
}

pub const PATH_INTERNAL_GET_ACCOUNT_STATE: &str = "/internal/get_account_state/:account_id";

#[utoipa::path(
    get,
    path = "/internal/get_account_state/{account_id}",
    params(AccountId),
    responses(
        (status = 200, description = "Get current account state", body = Account),
        (status = 500, description = "Internal server error or account ID was invalid"),
    ),
    security(),
)]
pub async fn internal_get_account_state<S: ReadData + GetUsers>(
    Path(account_id): Path<AccountId>,
    state: S,
) -> Result<Json<Account>, StatusCode> {
    let internal_id = state
        .users()
        .get_internal_id(account_id)
        .await
        .map_err(|e| {
            error!("Internal get account state error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    state
        .read()
        .account()
        .account(internal_id)
        .await
        .map(|account| account.into())
        .map_err(|e| {
            error!("Internal get account state error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
