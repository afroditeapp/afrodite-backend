use std::{net::SocketAddr, time::Duration};

use axum::{
    body::Body,
    extract::{ConnectInfo, FromRequest, State, rejection::JsonRejection},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use headers::{
    Authorization, CacheControl, ETag, HeaderMapExt, IfNoneMatch, authorization::Bearer,
};
use hyper::Request;
use model::AccessToken;
use serde::Serialize;
use server_data::app::GetConfig;
pub use server_state::utils::StatusCode;
use server_state::{StateForRouterCreation, app::GetAccessTokens};
use simple_backend::create_counters;
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .typed_get::<Authorization<Bearer>>()
        .ok_or(StatusCode::BAD_REQUEST)?;
    let key = AccessToken::new(header.token().to_string());

    if let Some((id, permissions, account_state)) =
        state.s.access_token_and_connection_exists(&key, addr).await
    {
        if state.allow_only_remote_bots {
            let is_remote_bot = state
                .s
                .config()
                .remote_bots()
                .iter()
                .any(|b| b.account_id() == id.as_id());
            if !is_remote_bot {
                API.access_token_found_not_remote_bot.incr();
                return Err(StatusCode::UNAUTHORIZED);
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
    access_token_found_not_remote_bot,
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
