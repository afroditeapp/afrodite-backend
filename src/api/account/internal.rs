//! Handlers for internal from Server to Server state transfers and messages

use axum::{Json, TypedHeader};

use hyper::StatusCode;

use crate::api::utils::ApiKeyHeader;

use super::{data::{AccountIdInternal, AccountIdLight}, GetApiKeys};

pub const PATH_CHECK_API_KEY: &str = "/internal/check_api_key";

#[utoipa::path(
    get,
    path = "/internal/check_api_key",
    responses(
        (status = 200, description = "Check API key", body = AccountIdLight),
        (status = 404, description = "API key was invalid"),
    ),
    security(("api_key" = [])),
)]
pub async fn check_api_key<S: GetApiKeys>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    state: S,
) -> Result<Json<AccountIdLight>, StatusCode> {
    state
        .api_keys()
        .api_key_exists(api_key.key())
        .await
        .ok_or(StatusCode::NOT_FOUND)
        .map(|id| id.as_light().into())
}
