use std::net::SocketAddr;

use axum::{
    extract::{rejection::JsonRejection, ConnectInfo, FromRequest},
    middleware::Next,
    response::{IntoResponse, Response},
};
use config::RUNNING_IN_DEBUG_MODE;
use headers::{Header, HeaderValue};
use hyper::{header, Request, StatusCode};
use model::AccessToken;
use serde::Serialize;
pub use utils::api::ACCESS_TOKEN_HEADER_STR;
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify,
};

use super::GetAccessTokens;

pub static API_KEY_HEADER: header::HeaderName = header::HeaderName::from_static(ACCESS_TOKEN_HEADER_STR);

pub async fn authenticate_with_api_key<T, S: GetAccessTokens>(
    state: S,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut req: Request<T>,
    next: Next<T>,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .get(ACCESS_TOKEN_HEADER_STR)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let key_str = header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
    let key = AccessToken::new(key_str.to_string());

    if let Some(id) = state
        .api_keys()
        .access_token_and_connection_exists(&key, addr)
        .await
    {
        req.extensions_mut().insert(id);
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub struct ApiKeyHeader(AccessToken);

impl ApiKeyHeader {
    pub fn key(&self) -> &AccessToken {
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
        Ok(ApiKeyHeader(AccessToken::new(value.to_string())))
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
                    ApiKeyValue::new(ACCESS_TOKEN_HEADER_STR),
                )),
            )
        }
    }
}

// Prevent axum from exposing API details in errors when not running in
// debug mode.

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct Json<T>(pub T);

impl<T> From<T> for Json<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl From<JsonRejection> for ApiError {
    fn from(value: JsonRejection) -> Self {
        Self {
            status: value.status(),
            message: value.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let json_error = if RUNNING_IN_DEBUG_MODE.value() {
            serde_json::json!({
                "status": self.status.as_u16(),
                "status_message": self.status.to_string(),
                "message": self.message,
            })
        } else {
            serde_json::json!({
                "status": self.status.as_u16(),
            })
        };

        (self.status, axum::Json(json_error)).into_response()
    }
}
