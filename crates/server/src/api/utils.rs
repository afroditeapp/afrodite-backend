use std::net::SocketAddr;

use axum::{extract::ConnectInfo, middleware::Next, response::Response};
use headers::{Header, HeaderValue};
use hyper::{header, Request, StatusCode};
use model::ApiKey;
pub use utils::api::API_KEY_HEADER_STR;
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify,
};

use super::GetApiKeys;
pub static API_KEY_HEADER: header::HeaderName = header::HeaderName::from_static(API_KEY_HEADER_STR);

pub async fn authenticate_with_api_key<T, S: GetApiKeys>(
    state: S,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut req: Request<T>,
    next: Next<T>,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .get(API_KEY_HEADER_STR)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let key_str = header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
    let key = ApiKey::new(key_str.to_string());

    if let Some(id) = state
        .api_keys()
        .api_key_and_connection_exists(&key, addr)
        .await
    {
        req.extensions_mut().insert(id);
        Ok(next.run(req).await)
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
