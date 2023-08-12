//! Handlers for internal from Server to Server state transfers and messages

use axum::{extract::Path, Json};
use hyper::StatusCode;
use tracing::error;

use model::{Account, AccountIdLight, ApiKey};

use crate::api::{GetApiKeys, GetUsers, ReadDatabase};

pub const PATH_INTERNAL_CHECK_API_KEY: &str = "/internal/check_api_key";

#[utoipa::path(
    get,
    path = "/internal/check_api_key",
    request_body(content = ApiKey),
    responses(
        (status = 200, description = "Check API key", body = AccountIdLight),
        (status = 404, description = "API key was invalid"),
    ),
    security(),
)]
pub async fn check_api_key<S: GetApiKeys>(
    Json(api_key): Json<ApiKey>,
    state: S,
) -> Result<Json<AccountIdLight>, StatusCode> {
    state
        .api_keys()
        .api_key_exists(&api_key)
        .await
        .ok_or(StatusCode::NOT_FOUND)
        .map(|id| id.as_light().into())
}

pub const PATH_INTERNAL_GET_ACCOUNT_STATE: &str = "/internal/get_account_state/:account_id";

#[utoipa::path(
    get,
    path = "/internal/get_account_state/{account_id}",
    params(AccountIdLight),
    responses(
        (status = 200, description = "Get current account state", body = Account),
        (status = 500, description = "Internal server error or account ID was invalid"),
    ),
    security(),
)]
pub async fn internal_get_account_state<S: ReadDatabase + GetUsers>(
    Path(account_id): Path<AccountIdLight>,
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
        .read_database()
        .read_json::<Account>(internal_id)
        .await
        .map(|account| account.into())
        .map_err(|e| {
            error!("Internal get account state error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
