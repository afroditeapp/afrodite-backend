pub mod data;
pub mod internal;

use axum::{extract::Path, middleware::Next, response::Response, Json, TypedHeader};
use headers::{Header, HeaderValue};
use hyper::{header, Request, StatusCode};
use tokio::sync::Mutex;
use utoipa::{
    openapi::security::{ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::server::session::AccountStateInRam;

use self::{
    data::{ApiKey, AccountId, Account, Capabilities, AccountIdLight, AccountSetup, AccountState},
};

use super::{get_account_id, GetConfig};

use tracing::error;

use super::{db_write, GetApiKeys, GetRouterDatabaseHandle, GetUsers, ReadDatabase, WriteDatabase, utils::ApiKeyHeader};

// TODO: Update register and login to support Apple and Google single sign on.

pub const PATH_REGISTER: &str = "/account/register";

#[utoipa::path(
    post,
    path = "/account/register",
    security(),
    responses(
        (status = 200, description = "New profile created.", body = [AccountId]),
        (status = 500, description = "Internal server error."),
    )
)]
pub async fn register<S: GetRouterDatabaseHandle + GetUsers + GetConfig>(
    state: S,
) -> Result<Json<AccountIdLight>, StatusCode> {
    // New unique UUID is generated every time so no special handling needed
    // to avoid database collisions.
    let id = AccountId::generate_new();

    let mut write_commands = state
        .database()
        .user_write_commands(&id);
    match write_commands.register(state.config()).await {
        Ok(()) => {
            state
                .users()
                .write()
                .await
                .insert(id.as_light(), Mutex::new(write_commands));
            Ok(id.as_light().into())
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
    request_body = AccountIdLight,
    responses(
        (status = 200, description = "Login successful.", body = [ApiKey]),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn login<S: GetApiKeys + WriteDatabase>(
    Json(id): Json<AccountIdLight>,
    state: S,
) -> Result<Json<ApiKey>, StatusCode> {

    let key = ApiKey::generate_new();

    db_write!(state, &id)?
        .await
        .update_current_api_key(&key)
        .await
        .map_err(|e| {
            error!("Login error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
        })?;

    let user_state = AccountStateInRam::new(id.to_full());
    state
        .api_keys()
        .write()
        .await
        .insert(key.clone(), user_state);

    Ok(key.into())
}


pub const PATH_ACCOUNT_STATE: &str = "/account/state";

#[utoipa::path(
    get,
    path = "/account/state",
    responses(
        (status = 200, description = "Request successfull.", body = [Account]),
        (status = 500, description = "Internal server error."),
    ),
    security(("api_key" = [])),
)]
pub async fn account_state<S: GetApiKeys + ReadDatabase>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    state: S,
) -> Result<Json<Account>, StatusCode> {
    let id = get_account_id!(state, api_key.key())?;

    state
        .read_database()
        .account(&id.to_full())
        .await
        .map(|account| account.into())
        .map_err(|e| {
            error!("Get profile error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
        })
}


pub const PATH_ACCOUNT_SETUP: &str = "/account/setup";

/// Setup non-changeable user information during `initial setup` state.
#[utoipa::path(
    post,
    path = "/account/setup",
    responses(
        (status = 200, description = "Request successfull.", body = [Account]),
        (
            status = 500,
            description = "Account state is not initial setup or some other error"),
    ),
    security(("api_key" = [])),
)]
pub async fn account_setup<S: GetApiKeys + ReadDatabase + WriteDatabase>(
    TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
    Json(data): Json<AccountSetup>,
    state: S,
) -> Result<StatusCode, StatusCode> {
    let id = get_account_id!(state, api_key.key())?;

    let account = state
        .read_database()
        .account(&id.to_full())
        .await
        .map_err(|e| {
            error!("Get profile error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
        })?;

    if account.state() == AccountState::InitialSetup {
        db_write!(state, &id)?
            .await
            .update_json(&data)
            .await
            .map_err(|e| {
                error!("Write database error: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
            })
            .map(|_| StatusCode::OK)
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
