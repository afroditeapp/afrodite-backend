pub mod data;
pub mod internal;

use std::sync::Arc;

use axum::{Json, TypedHeader};

use hyper::StatusCode;

use crate::server::session::AccountStateInRam;

use self::data::{Account, AccountId, AccountIdLight, AccountSetup, AccountState, ApiKey};

use super::{GetConfig, utils::{get_account, get_account_from_api_key}};

use tracing::error;

use super::{
    utils::ApiKeyHeader, GetApiKeys, GetRouterDatabaseHandle, GetUsers, ReadDatabase,
    WriteDatabase,
};

// TODO: Update register and login to support Apple and Google single sign on.

pub const PATH_REGISTER: &str = "/account/register";

/// Register new account. Returns new account ID which is UUID.
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

    let register = state
        .database()
        .register(id.as_light(), state.config());
    match register.await {
        Ok(internal_id) => {
            let account_state = AccountStateInRam::new(internal_id);
            state
                .users()
                .write()
                .await
                .insert(id.as_light(), Arc::new(account_state));
            Ok(id.as_light().into())
        }
        Err(e) => {
            error!("Error: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub const PATH_LOGIN: &str = "/account/login";

/// Get new refresh token and ApiKey.
#[utoipa::path(
    post,
    path = "/account/login",
    security(),
    request_body = AccountIdLight,
    responses(
        (status = 200, description = "Login successful.", body = [AuthPair]),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn login<S: GetApiKeys + WriteDatabase + GetUsers>(
    Json(id): Json<AccountIdLight>,
    state: S,
) -> Result<Json<ApiKey>, StatusCode> {
    let key = ApiKey::generate_new();
    let account_state = get_account(&state, id, |account| account.clone()).await?;

    state
        .write_database()
        .update_api_key(account_state.id(), Some(&key))
        .await
        .map_err(|e| {
            error!("Login error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
        })?;

    state
        .api_keys()
        .write()
        .await
        .insert(key.clone(), account_state);

    Ok(key.into())
}

pub const PATH_API_KEY: &str = "/account/api_key";

/// Get app refresh token which is used for getting new API keys.
#[utoipa::path(
    post,
    path = "/account/api_key",
    security(),
    request_body = AccountIdLight,
    responses(
        (status = 200, description = "Login successful.", body = [ApiKey]),
        (status = 500, description = "Internal server error."),
    ),
)]
pub async fn api_key<S: GetApiKeys + WriteDatabase + GetUsers>(
    Json(id): Json<AccountIdLight>,
    state: S,
) -> Result<Json<ApiKey>, StatusCode> {
    let key = ApiKey::generate_new();
    let account_state = get_account(&state, id, |account| account.clone()).await?;

    todo!()
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
    let id = get_account_from_api_key(&state, api_key.key(), |a| a.id()).await?;

    state
        .read_database()
        .read_json::<Account>(id)
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
    let id = get_account_from_api_key(&state, api_key.key(), |a| a.id()).await?;

    let account = state
        .read_database()
        .read_json::<Account>(id)
        .await
        .map_err(|e| {
            error!("Get profile error: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR // Database reading failed.
        })?;

    if account.state() == AccountState::InitialSetup {
        state.write_database()
            .update_json(id, &data)
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
