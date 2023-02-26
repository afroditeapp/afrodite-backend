pub mod profile;
pub mod user;
pub mod internal;

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
    profile::Profile,
    user::{ApiKey, UserId},
};

use tracing::error;

use super::{db_write, GetApiKeys, GetRouterDatabaseHandle, GetUsers, ReadDatabase, WriteDatabase};

// TODO: Add timeout for database commands

pub const PATH_REGISTER: &str = "/register";

#[utoipa::path(
    post,
    path = "/register",
    security(),
    responses(
        (status = 200, description = "New profile created.", body = [UserId]),
        (status = 500),
    )
)]
pub async fn register<S: GetRouterDatabaseHandle + GetUsers>(
    state: S,
) -> Result<Json<UserId>, StatusCode> {
    // New unique UUID is generated every time so no special handling needed.
    let new_user_id = UserId::new(uuid::Uuid::new_v4().simple().to_string());

    let mut write_commands = state.database().user_write_commands(&new_user_id);
    match write_commands.register().await {
        Ok(()) => {
            state
                .users()
                .write()
                .await
                .insert(new_user_id.clone(), Mutex::new(write_commands));
            Ok(new_user_id.into())
        }
        Err(e) => {
            error!("Error: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub const PATH_LOGIN: &str = "/login";

#[utoipa::path(
    post,
    path = "/login",
    security(),
    request_body = UserId,
    responses(
        (status = 200, description = "Login successful.", body = [ApiKey]),
        (status = 500),
    ),
)]
pub async fn login<S: GetApiKeys + WriteDatabase>(
    Json(user_id): Json<UserId>,
    state: S,
) -> Result<Json<ApiKey>, StatusCode> {
    // TODO: check that UserId contains only hexadecimals

    let key = ApiKey::new(uuid::Uuid::new_v4().simple().to_string());

    db_write!(state, &user_id)?
        .await
        .update_current_api_key(&key)
        .await
        .map_err(|e| {
            error!("Login error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
        })?;

    let user_state = UserState::new(user_id);
    state
        .api_keys()
        .write()
        .await
        .insert(key.clone(), user_state);

    Ok(key.into())
}

pub const PATH_GET_PROFILE: &str = "/profile/:user_id";

#[utoipa::path(
    get,
    path = "/profile/{user_id}",
    params(UserId),
    responses(
        (status = 200, description = "Get profile.", body = [Profile]),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_profile<S: ReadDatabase>(
    Path(user_id): Path<UserId>,
    state: S,
) -> Result<Json<Profile>, StatusCode> {
    // TODO: Validate user id
    state
        .read_database()
        .user_profile(&user_id)
        .await
        .map(|profile| profile.into())
        .map_err(|e| {
            error!("Get profile error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
        })
}

pub const PATH_POST_PROFILE: &str = "/profile";

#[utoipa::path(
    post,
    path = "/profile",
    request_body = Profile,
    responses(
        (status = 200, description = "Update profile", body = [Profile]),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn post_profile<S: GetApiKeys + WriteDatabase>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Json(profile): Json<Profile>,
    state: S,
) -> Result<(), StatusCode> {
    let keys = state.api_keys().read().await;
    let user_id = keys.get(&api_key.0).ok_or(StatusCode::UNAUTHORIZED)?.id();

    db_write!(state, user_id)?
        .await
        .update_user_profile(&profile)
        .await
        .map_err(|e| {
            error!("Post profile error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
        })?;

    Ok(())
}

pub const API_KEY_HEADER_STR: &str = "x-api-key";
pub static API_KEY_HEADER: header::HeaderName = header::HeaderName::from_static(API_KEY_HEADER_STR);

pub async fn authenticate_core_api<T, S: GetApiKeys>(
    state: S,
    req: Request<T>,
    next: Next<T>,
) -> Result<Response, StatusCode> {
    let header = req
        .headers()
        .get(API_KEY_HEADER_STR)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let key_str = header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
    let key = ApiKey::new(key_str.to_string());

    if state.api_keys().read().await.contains_key(&key) {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub struct ApiKeyHeader(ApiKey);

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
