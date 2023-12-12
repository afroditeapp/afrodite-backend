//! Handlers for internal from Server to Server state transfers and messages

use axum::extract::Path;
use model::{AccessToken, Account, AccountId};

use crate::{
    api::utils::{Json, StatusCode},
    app::{GetAccessTokens, GetAccounts, ReadData},
    perf::ACCOUNT_INTERNAL,
};

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
pub async fn check_access_token<S: GetAccessTokens>(
    Json(token): Json<AccessToken>,
    state: S,
) -> Result<Json<AccountId>, StatusCode> {
    ACCOUNT_INTERNAL.check_access_token.incr();
    state
        .access_tokens()
        .access_token_exists(&token)
        .await
        .ok_or(StatusCode::NOT_FOUND)
        .map(|id| id.as_id().into())
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
pub async fn internal_get_account_state<S: ReadData + GetAccounts>(
    Path(account_id): Path<AccountId>,
    state: S,
) -> Result<Json<Account>, StatusCode> {
    ACCOUNT_INTERNAL.internal_get_account_state.incr();
    let internal_id = state.accounts().get_internal_id(account_id).await?;

    let account = state.read().account().account(internal_id).await?;

    Ok(account.into())
}
