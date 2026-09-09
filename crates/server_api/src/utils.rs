use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use axum::{
    body::Body,
    extract::{ConnectInfo, FromRequest, FromRequestParts, State, rejection::JsonRejection},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use headers::{
    Authorization, CacheControl, ETag, HeaderMapExt, IfNoneMatch, authorization::Bearer,
};
use http::request::Parts;
use hyper::Request;
use model::AccessToken;
use serde::Serialize;
pub use server_common::data::connection_id::ConnectionId;
use server_data::{
    app::{GetConfig, ReadData},
    read::GetReadCommandsCommon,
};
pub use server_state::utils::StatusCode;
use server_state::{StateForRouterCreation, app::GetAccessTokens};
use simple_backend::{create_counters, utils::client_ip_from_http_header_if_possible};
use simple_backend_config::RUNNING_IN_DEBUG_MODE;
use utoipa::{
    Modify,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

/// Middleware for authenticating requests with access tokens.
///
/// Adds `AccountIdInternal` extension to request, so that adding
/// "Extension(api_caller_account_id): Extension<AccountIdInternal>"
/// to handlers is possible.
///
/// Adds `Permissions` extension to request, so that adding
/// "Extension(api_caller_permissions): Extension<Permissions>"
/// to handlers is possible.
///
/// Adds `AccountState` extension to request, so that adding
/// "Extension(api_caller_account_state): Extension<AccountState>"
/// to handlers is possible.
pub async fn authenticate_with_access_token(
    State(state): State<StateForRouterCreation>,
    ClientIp(addr): ClientIp,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .typed_get::<Authorization<Bearer>>()
        .ok_or(StatusCode::BAD_REQUEST)?;
    let key = AccessToken::new(header.token().to_string());

    if let Some((id, permissions, account_state)) =
        state.s.access_token_and_ip_is_valid(&key, addr).await
    {
        if state.allow_only_bots {
            match state.s.read().common().is_bot(id).await {
                Ok(true) => (),
                _ => return Err(StatusCode::UNAUTHORIZED),
            }
        }

        API.access_token_found.incr();
        req.extensions_mut().insert(id);
        req.extensions_mut().insert(permissions);
        req.extensions_mut().insert(account_state);
        Ok(next.run(req).await)
    } else {
        API.access_token_not_found.incr();
        Err(StatusCode::UNAUTHORIZED)
    }
}

create_counters!(
    ApiCounters,
    API,
    API_COUNTERS_LIST,
    access_token_found,
    access_token_not_found,
);

/// Utoipa API doc security config
pub struct SecurityApiAccessTokenDefault;

impl Modify for SecurityApiAccessTokenDefault {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "access_token",
                SecurityScheme::Http(Http::builder().scheme(HttpAuthScheme::Bearer).into()),
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

/// Extractor which returns only the client's IP address.
///
/// This is an alternative to [`ConnectInfo<SocketAddr>`] for setups where the
/// server is behind a reverse proxy (such as Caddy) which terminates TLS.
///
/// When the server's public API TLS is disabled (see
/// `[tls.public_api] disable = true`), the real client IP address is only
/// available via the `X-Forwarded-For` HTTP header set by the reverse proxy.
///
/// In that case the rightmost IP address in the `X-Forwarded-For` header is
/// used, as it is the one added by the trusted reverse proxy closest to the
/// server. When TLS is not disabled, the header is ignored and the IP address
/// from the actual TCP connection is used instead.
#[derive(Debug, Clone, Copy)]
pub struct ClientIp(pub IpAddr);

impl<S> FromRequestParts<S> for ClientIp
where
    S: Send + Sync + GetConfig,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let connect_info = ConnectInfo::<SocketAddr>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let ip = if state.config().simple_backend().public_api_tls_disabled() {
            client_ip_from_http_header_if_possible(&parts.headers, *connect_info)
        } else {
            connect_info.ip()
        };

        Ok(Self(ip))
    }
}

/// Extractor which returns the client's IP address and port number as a
/// [`ConnectionId`].
///
/// This is an alternative to [`ConnectInfo<SocketAddr>`] for setups where the
/// server is behind a reverse proxy (such as Caddy) which terminates TLS.
///
/// When the server's public API TLS is disabled (see
/// `[tls.public_api] disable = true`), the real client IP address is only
/// available via the `X-Forwarded-For` HTTP header set by the reverse proxy.
///
/// In that case the rightmost IP address in the `X-Forwarded-For` header is
/// used, as it is the one added by the trusted reverse proxy closest to the
/// server. When TLS is not disabled, the header is ignored and the IP address
/// from the actual TCP connection is used instead. The port number is always
/// taken from the actual TCP connection.
#[derive(Debug, Clone, Copy)]
pub struct ClientConnectionId(pub ConnectionId);

impl<S> FromRequestParts<S> for ClientConnectionId
where
    S: Send + Sync + GetConfig,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let connect_info = ConnectInfo::<SocketAddr>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let ip = if state.config().simple_backend().public_api_tls_disabled() {
            client_ip_from_http_header_if_possible(&parts.headers, *connect_info)
        } else {
            connect_info.ip()
        };

        Ok(Self(ConnectionId::new(ip, connect_info.port())))
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

pub fn cache_control_for_images() -> CacheControl {
    const MONTH_SECONDS: u64 = 60 * 60 * 24 * 30;
    CacheControl::new()
        .with_max_age(Duration::from_secs(MONTH_SECONDS * 3))
        .with_must_revalidate()
        .with_private()
        .with_immutable()
}

pub fn cache_control_for_images_with_downgraded_quality() -> CacheControl {
    const HOUR_SECONDS: u64 = 60 * 60;
    CacheControl::new()
        .with_max_age(Duration::from_secs(HOUR_SECONDS * 3))
        .with_must_revalidate()
        .with_private()
        .with_immutable()
}

pub trait IfNoneMatchExtensions {
    fn matches(&self, tag: &ETag) -> bool;
}

impl IfNoneMatchExtensions for IfNoneMatch {
    fn matches(&self, tag: &ETag) -> bool {
        !self.precondition_passes(tag)
    }
}

impl IfNoneMatchExtensions for Option<TypedHeader<IfNoneMatch>> {
    fn matches(&self, tag: &ETag) -> bool {
        if let Some(browser_etag) = self {
            browser_etag.matches(tag)
        } else {
            false
        }
    }
}
