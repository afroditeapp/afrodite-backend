//! Handlers for internal from Server to Server state transfers and messages

use axum::{extract::Path, middleware::Next, response::Response, Json, TypedHeader};
use headers::{Header, HeaderValue};
use hyper::{header, Request, StatusCode};
use tokio::sync::Mutex;
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::server::session::UserState;

use self::{
    super::super::model::{Profile, ApiKey, AccountId},
};

use self::super::super::media::image::ImageFileName;

use tracing::error;

use crate::api::utils::ApiKeyHeader;

use super::{
    db_write, GetApiKeys, GetRouterDatabaseHandle, GetUsers, ReadDatabase,
    WriteDatabase,
};

pub const PATH_CHECK_API_KEY: &str = "/internal/check_api_key";

#[utoipa::path(
    get,
    path = "/internal/check_api_key",
    responses(
        (status = 200, description = "Check API key", body = [AccountId]),
        (status = 404, description = "API key was invalid"),
    ),
    security(("api_key" = [])),
)]
pub async fn check_api_key<S: GetApiKeys>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    state: S,
) -> Result<Json<AccountId>, StatusCode> {
    state
        .api_keys()
        .read()
        .await
        .get(api_key.key())
        .ok_or(StatusCode::NOT_FOUND)
        .map(|state| state.id().clone().into())
}
