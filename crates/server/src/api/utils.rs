use std::net::SocketAddr;

use axum::{
    body::Body, extract::{rejection::JsonRejection, ConnectInfo, FromRequest, State}, middleware::Next, response::{IntoResponse, Response}
};
use config::file::ConfigFileError;
use headers::{Header, HeaderValue};
use hyper::{header, Request};
use model::AccessToken;
use serde::Serialize;
use simple_backend::{
    manager_client::ManagerClientError,
    sign_in_with::{apple::SignInWithAppleError, google::SignInWithGoogleError},
};
use simple_backend_config::RUNNING_IN_DEBUG_MODE;
pub use utils::api::ACCESS_TOKEN_HEADER_STR;
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify,
};

use crate::{
    app::{GetAccessTokens, ReadData},
    data::{cache::CacheError, DataError},
    event::EventError,
    internal::InternalApiError,
};

pub static ACCESS_TOKEN_HEADER: header::HeaderName =
    header::HeaderName::from_static(ACCESS_TOKEN_HEADER_STR);

/// Middleware for authenticating requests with access tokens.
///
/// Adds `AccountIdInternal` extension to request, so that adding
/// "Extension(api_caller_account_id): Extension<AccountIdInternal>"
/// to handlers is possible.
///
/// Adds `Capabilities` extension to request, so that adding
/// "Extension(api_caller_capabilities): Extension<Capabilities>"
/// to handlers is possible.
pub async fn authenticate_with_access_token<S: GetAccessTokens + ReadData>(
    State(state): State<S>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .get(ACCESS_TOKEN_HEADER_STR)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let key_str = header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
    let key = AccessToken::new(key_str.to_string());

    if let Some((id, capabilities)) = state
        .access_tokens()
        .access_token_and_connection_exists(&key, addr)
        .await
    {
        req.extensions_mut().insert(id);
        req.extensions_mut().insert(capabilities);
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub struct AccessTokenHeader(AccessToken);

impl AccessTokenHeader {
    pub fn key(&self) -> &AccessToken {
        &self.0
    }
}

impl Header for AccessTokenHeader {
    fn name() -> &'static headers::HeaderName {
        &ACCESS_TOKEN_HEADER
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i headers::HeaderValue>,
    {
        let value = values.next().ok_or_else(headers::Error::invalid)?;
        let value = value.to_str().map_err(|_| headers::Error::invalid())?;
        Ok(AccessTokenHeader(AccessToken::new(value.to_string())))
    }

    fn encode<E: Extend<headers::HeaderValue>>(&self, values: &mut E) {
        let header = HeaderValue::from_str(self.0.as_str()).unwrap();
        values.extend(std::iter::once(header))
    }
}

/// Utoipa API doc security config
pub struct SecurityApiAccessTokenDefault;

impl Modify for SecurityApiAccessTokenDefault {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "access_token",
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
    status: hyper::StatusCode,
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
    #[error("Data reading or writing failed")]
    Data,
    #[error("Cache reading or writing failed")]
    Cache,
    #[error("Sign in with Google error")]
    SignInWithGoogle,
    #[error("Sign in with Apple error")]
    SignInWithApple,
    #[error("Internal API error")]
    InternalApiError,
    #[error("Manager client error")]
    ManagerClientError,
    #[error("Config file error")]
    ConfigFileError,
    #[error("Event error")]
    EventError,
    #[error("Content processing error")]
    ContentProcessingError,
}

impl From<error_stack::Report<DataError>> for StatusCode {
    #[track_caller]
    fn from(value: error_stack::Report<DataError>) -> Self {
        tracing::error!("{:?}", value.change_context(RequestError::Data));
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<error_stack::Report<CacheError>> for StatusCode {
    #[track_caller]
    fn from(value: error_stack::Report<CacheError>) -> Self {
        tracing::error!("{:?}", value.change_context(RequestError::Cache));
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<error_stack::Report<SignInWithGoogleError>> for StatusCode {
    #[track_caller]
    fn from(value: error_stack::Report<SignInWithGoogleError>) -> Self {
        tracing::error!("{:?}", value.change_context(RequestError::SignInWithGoogle));
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<error_stack::Report<SignInWithAppleError>> for StatusCode {
    #[track_caller]
    fn from(value: error_stack::Report<SignInWithAppleError>) -> Self {
        tracing::error!("{:?}", value.change_context(RequestError::SignInWithApple));
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<error_stack::Report<InternalApiError>> for StatusCode {
    #[track_caller]
    fn from(value: error_stack::Report<InternalApiError>) -> Self {
        tracing::error!("{:?}", value.change_context(RequestError::InternalApiError));
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<error_stack::Report<ManagerClientError>> for StatusCode {
    #[track_caller]
    fn from(value: error_stack::Report<ManagerClientError>) -> Self {
        tracing::error!(
            "{:?}",
            value.change_context(RequestError::ManagerClientError)
        );
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<error_stack::Report<ConfigFileError>> for StatusCode {
    #[track_caller]
    fn from(value: error_stack::Report<ConfigFileError>) -> Self {
        tracing::error!("{:?}", value.change_context(RequestError::ConfigFileError));
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<error_stack::Report<EventError>> for StatusCode {
    #[track_caller]
    fn from(value: error_stack::Report<EventError>) -> Self {
        tracing::error!("{:?}", value.change_context(RequestError::EventError));
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

// pub trait IntoStatusCodeError<Ok, Err: Into<StatusCode>>: Sized {
//     fn into_status_error(self) -> std::result::Result<Ok, StatusCode>;
// }

// impl <Ok, Err: Into<StatusCode>> IntoStatusCodeError<Ok, Err> for std::result::Result<Ok, Err> {
//     fn into_status_error(self) -> std::result::Result<Ok, StatusCode> {
//         self.map_err(|err| err.into())
//     }
// }
