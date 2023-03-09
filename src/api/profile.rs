pub mod data;

use axum::{extract::Path, middleware::Next, response::Response, Json, TypedHeader};
use headers::{Header, HeaderValue};
use hyper::{header, Request, StatusCode};
use tokio::sync::Mutex;
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::server::session::AccountState;

use self::data::Profile;

use super::{model::{
    Account,
    ApiKey, AccountId, AccountIdLight
}, get_account_id};

use tracing::error;

use super::{db_write, GetApiKeys, GetRouterDatabaseHandle, GetUsers, ReadDatabase, WriteDatabase, utils::ApiKeyHeader};

// TODO: Add timeout for database commands


pub const PATH_GET_PROFILE: &str = "/profile/:account_id";

#[utoipa::path(
    get,
    path = "/profile/{account_id}",
    params(AccountIdLight),
    responses(
        (status = 200, description = "Get profile.", body = [Profile]),
        (status = 500),
    ),
    security(("api_key" = [])),
)]
pub async fn get_profile<S: ReadDatabase>(
    Path(account_id): Path<AccountIdLight>,
    state: S,
) -> Result<Json<Profile>, StatusCode> {
    // TODO: Validate user id
    state
        .read_database()
        .user_profile(&account_id.to_full())
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
    let id = get_account_id!(state, api_key.key())?;

    db_write!(state, id)?
        .await
        .update_user_profile(&profile)
        .await
        .map_err(|e| {
            error!("Post profile error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
        })?;

    Ok(())
}
