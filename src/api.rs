//! HTTP API types and request handlers for all servers.

// Routes
pub mod account;
pub mod profile;
pub mod media;

pub mod model;
pub mod utils;

use std::collections::HashMap;

use tokio::sync::{Mutex, RwLock};
use utoipa::{
    Modify, OpenApi,
};

use crate::{server::{
    database::{ write::WriteCommands, RouterDatabaseHandle, current::read::SqliteReadCommands},
    session::{SessionManager, AccountStateInRam},
}, client::{account::AccountInternalApi, media::MediaInternalApi}, config::Config};

use self::model::{
    ApiKey, AccountIdLight,
};

use utils::SecurityApiTokenDefault;

// Paths

pub const PATH_PREFIX: &str = "/api/v1/";

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        account::register,
        account::login,
        account::account_state,
        account::account_setup,
        account::internal::check_api_key,
        profile::get_profile,
        profile::post_profile,
        media::get_image,
        media::internal::post_image,
    ),
    components(schemas(
        account::data::AccountId,
        account::data::AccountIdLight,
        account::data::ApiKey,
        account::data::Account,
        account::data::AccountState,
        account::data::AccountSetup,
        account::data::Capabilities,
        profile::data::Profile,
        media::data::ImageFileName,
        media::data::ImageFile,
    )),
    modifiers(&SecurityApiTokenDefault),
)]
pub struct ApiDoc;



// App state getters

pub trait GetSessionManager {
    fn session_manager(&self) -> &SessionManager;
}

pub trait GetRouterDatabaseHandle {
    fn database(&self) -> &RouterDatabaseHandle;
}

pub trait GetApiKeys {
    /// Users which are logged in.
    fn api_keys(&self) -> &RwLock<HashMap<ApiKey, AccountStateInRam>>;
}

pub trait GetUsers {
    /// All users registered in the service.
    fn users(&self) -> &RwLock<HashMap<AccountIdLight, Mutex<WriteCommands>>>;
}

/// Use with db_write macro.
pub trait WriteDatabase {
    fn write_database_with_db_macro_do_not_call_this_outside_macros(
        &self,
    ) -> &RwLock<HashMap<AccountIdLight, Mutex<WriteCommands>>>;
}

pub trait ReadDatabase {
    fn read_database(&self) -> SqliteReadCommands<'_>;
}

pub trait GetCoreServerInternalApi {
    fn core_server_internal_api(&self) -> AccountInternalApi;
}

pub trait GetMediaServerInternalApi {
    fn media_server_internal_api(&self) -> MediaInternalApi;
}

pub trait GetConfig {
    fn config(&self) -> &Config;
}

/// Helper macro for getting write access to database.
///
/// Might make return error StatusCode::INTERNAL_SERVER_ERROR
/// if AccountId does not exist.
///
///
/// Example usage:
///
/// ```rust
/// pub async fn axum_route_handler<S: WriteDatabase>(
///     state: S,
/// ) -> Result<(), StatusCode> {
///     let key = ApiKey::new(uuid::Uuid::new_v4().simple().to_string());
///
///     db_write!(state, &user_id)
///         .update_current_api_key(&key)
///         .await
///         .map_err(|e| {
///             error!("Login error: {e:?}");
///             StatusCode::INTERNAL_SERVER_ERROR // Database writing failed.
///         })?;
///     Ok(())
/// }
/// ```
macro_rules! db_write {
    ($users:expr, $user_id:expr) => {
        $users
            .write_database_with_db_macro_do_not_call_this_outside_macros()
            .read()
            .await
            .get(&$user_id)
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR) // User does not exists
            .map(|x| async { x.lock().await })
    };
}


/// Helper macro for converting ApiKey to AccountId.
///
/// Might make return error StatusCode::INTERNAL_SERVER_ERROR
/// if ApiKey does not exist.
///
/// Example usage:
///
/// ```rust
/// pub async fn axum_route_handler<S: GetApiKey>(
///     TypedHeader(api_key): TypedHeader<ApiKeyHeader>,
///     state: S,
/// ) -> Result<(), StatusCode> {
///     let id = get_account_id!(state, api_key.key())?;
///     Ok(())
/// }
/// ```
macro_rules! get_account_id {
    ($all_keys:expr, $api_key:expr) => {
        $all_keys
            .api_keys()
            .read()
            .await
            .get($api_key)
            .ok_or(StatusCode::UNAUTHORIZED)
            .map(|x| x.id().as_light())
    };
}

pub(crate) use db_write;
pub(crate) use get_account_id;
