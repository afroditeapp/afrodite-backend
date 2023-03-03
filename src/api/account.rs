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
    user::{ApiKey, AccountId},
};

use tracing::error;

use super::{db_write, GetApiKeys, GetRouterDatabaseHandle, GetUsers, ReadDatabase, WriteDatabase};


pub const PATH_REGISTER: &str = "/register";

#[utoipa::path(
    post,
    path = "/register",
    security(),
    responses(
        (status = 200, description = "New profile created.", body = [AccountId]),
        (status = 500),
    )
)]
pub async fn register<S: GetRouterDatabaseHandle + GetUsers>(
    state: S,
) -> Result<Json<AccountId>, StatusCode> {
    // New unique UUID is generated every time so no special handling needed.
    let new_user_id = AccountId::new(uuid::Uuid::new_v4().simple().to_string());

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
    request_body = AccountId,
    responses(
        (status = 200, description = "Login successful.", body = [ApiKey]),
        (status = 500),
    ),
)]
pub async fn login<S: GetApiKeys + WriteDatabase>(
    Json(user_id): Json<AccountId>,
    state: S,
) -> Result<Json<ApiKey>, StatusCode> {
    // TODO: check that AccountId contains only hexadecimals

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
