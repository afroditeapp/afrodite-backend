use std::{
    net::SocketAddr,
    sync::atomic::{AtomicBool, Ordering},
};

use axum::{
    body::Body,
    extract::ConnectInfo,
    middleware::Next,
    response::{IntoResponse, Response},
};
use headers::{Header, HeaderValue};
use hyper::{header, Request};
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify,
};

use super::GetConfig;
use crate::{
    config::GetConfigError,
    server::{client::ApiError, info::SystemInfoError, update::UpdateError},
};

/// If true then password has been guessed and manager API is now locked.
static API_SECURITY_LOCK: AtomicBool = AtomicBool::new(false);

pub const API_KEY_HEADER_STR: &str = "x-api-key";
pub static API_KEY_HEADER: header::HeaderName = header::HeaderName::from_static(API_KEY_HEADER_STR);

pub async fn authenticate_with_api_key<S: GetConfig>(
    state: S,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .get(API_KEY_HEADER_STR)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let key_str = header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;

    if API_SECURITY_LOCK.load(Ordering::Relaxed) {
        Err(StatusCode::LOCKED)
    } else if state.config().api_key() != key_str {
        API_SECURITY_LOCK.store(true, Ordering::Relaxed);
        tracing::error!(
            "API key has been guessed. API is now locked. Guesser information, addr: {}",
            addr
        );
        Err(StatusCode::LOCKED)
    } else {
        Ok(next.run(req).await)
    }
}

pub struct ApiKeyHeader(String);

impl ApiKeyHeader {
    pub fn key(&self) -> &String {
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
        Ok(ApiKeyHeader(value.to_string()))
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

#[allow(non_camel_case_types)]
pub enum StatusCode {
    /// 400
    BAD_REQUEST,
    /// 401
    UNAUTHORIZED,
    /// 500
    INTERNAL_SERVER_ERROR,
    /// 406
    NOT_ACCEPTABLE,
    /// 404
    NOT_FOUND,
    /// 304
    NOT_MODIFIED,
    /// 423
    LOCKED,
}

impl From<StatusCode> for hyper::StatusCode {
    fn from(value: StatusCode) -> Self {
        match value {
            StatusCode::BAD_REQUEST => hyper::StatusCode::BAD_REQUEST,
            StatusCode::UNAUTHORIZED => hyper::StatusCode::UNAUTHORIZED,
            StatusCode::INTERNAL_SERVER_ERROR => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::NOT_ACCEPTABLE => hyper::StatusCode::NOT_ACCEPTABLE,
            StatusCode::NOT_FOUND => hyper::StatusCode::NOT_FOUND,
            StatusCode::NOT_MODIFIED => hyper::StatusCode::NOT_MODIFIED,
            StatusCode::LOCKED => hyper::StatusCode::LOCKED,
        }
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        let status: hyper::StatusCode = self.into();
        status.into_response()
    }
}

#[derive(thiserror::Error, Debug)]
enum RequestError {
    #[error("Update manager error")]
    Update,

    #[error("API error")]
    Api,

    #[error("Config error")]
    Config,

    #[error("System info error")]
    SystemInfo,
}

impl From<error_stack::Report<UpdateError>> for StatusCode {
    #[track_caller]
    fn from(value: error_stack::Report<UpdateError>) -> Self {
        tracing::error!("{:?}", value.change_context(RequestError::Update));
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<error_stack::Report<ApiError>> for StatusCode {
    #[track_caller]
    fn from(value: error_stack::Report<ApiError>) -> Self {
        tracing::error!("{:?}", value.change_context(RequestError::Api));
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<error_stack::Report<GetConfigError>> for StatusCode {
    #[track_caller]
    fn from(value: error_stack::Report<GetConfigError>) -> Self {
        tracing::error!("{:?}", value.change_context(RequestError::Config));
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<error_stack::Report<SystemInfoError>> for StatusCode {
    #[track_caller]
    fn from(value: error_stack::Report<SystemInfoError>) -> Self {
        tracing::error!("{:?}", value.change_context(RequestError::SystemInfo));
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
