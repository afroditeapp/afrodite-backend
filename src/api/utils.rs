use axum::{extract::Path, middleware::Next, response::Response, Json, TypedHeader};
use headers::{Header, HeaderValue};
use hyper::{header, Request, StatusCode};
use tokio::sync::Mutex;
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::server::session::AccountState;

use super::{model::{
    Profile,
    ApiKey, AccountId
}, GetCoreServerInternalApi, GetConfig};

use tracing::error;

use super::{db_write, GetApiKeys, GetRouterDatabaseHandle, GetUsers, ReadDatabase, WriteDatabase};

pub const API_KEY_HEADER_STR: &str = "x-api-key";
pub static API_KEY_HEADER: header::HeaderName = header::HeaderName::from_static(API_KEY_HEADER_STR);

pub async fn authenticate_with_api_key<T, S: GetApiKeys + GetCoreServerInternalApi + GetConfig>(
    state: S,
    req: Request<T>,
    next: Next<T>,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .get(API_KEY_HEADER_STR)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let key_str =
        header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
    let key = ApiKey::new(key_str.to_string());

    if state.api_keys().read().await.contains_key(&key) {
        Ok(next.run(req).await)
    } else if !state.config().components().account {
        // Check ApiKey from external service

        match state.core_server_internal_api().check_api_key(key).await {
            Ok(Some(user_id)) => {
                // TODO: Cache this API key.
                Ok(next.run(req).await)
            }
            Ok(None) => Err(StatusCode::UNAUTHORIZED),
            Err(e) => {
                // NOTE: Logging every error is not good as it would spam
                // the log, but maybe an error counter or logging just
                // once for a while.
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub struct ApiKeyHeader(ApiKey);

impl ApiKeyHeader {
    pub fn key(&self) -> &ApiKey {
        &self.0
    }
}

impl Header for ApiKeyHeader {
    fn name() -> &'static headers::HeaderName {
        &API_KEY_HEADER
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        let value = value.to_str().map_err(|_| headers::Error::invalid())?;
        Ok(ApiKeyHeader(ApiKey::new(value.to_string())))
    }

    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        let header = HeaderValue::from_str(self.0.as_str()).unwrap();
        values.extend(std::iter::once(header))
    }
}

/// Utoipa API doc security config
pub struct SecurityApiTokenDefault;

impl Modify for SecurityApiTokenDefault {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(utoipa::openapi::security::ApiKey::Header(
                    ApiKeyValue::new(API_KEY_HEADER_STR),
                )),
            )
        }
    }
}
