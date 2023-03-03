pub mod profile;

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
    super::account::user::{ApiKey, UserId},
};

use tracing::error;

use super::{db_write, GetApiKeys, GetRouterDatabaseHandle, GetUsers, ReadDatabase, WriteDatabase, utils::ApiKeyHeader};

// TODO: Add timeout for database commands


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
    let user_id = keys.get(api_key.key()).ok_or(StatusCode::UNAUTHORIZED)?.id();

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
