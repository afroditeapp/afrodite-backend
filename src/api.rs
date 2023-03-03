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
    openapi::{
        self,
        security::{ApiKeyValue, SecurityScheme},
    },
    Modify, OpenApi,
};

use crate::server::{
    database::{read::ReadCommands, write::WriteCommands, RouterDatabaseHandle},
    internal::{CoreServerInternalApi, MediaServerInternalApi},
    session::{SessionManager, UserState},
};

use self::model::{
    ApiKey, AccountId,
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
        account::internal::check_api_key,
        profile::get_profile,
        profile::post_profile,
        media::get_image,
        media::internal::post_image,
    ),
    components(schemas(
        account::user::AccountId,
        account::user::ApiKey,
        profile::profile::Profile,
        media::image::ImageFileName,
        media::image::ImageFile,
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
    fn api_keys(&self) -> &RwLock<HashMap<ApiKey, UserState>>;
}

pub trait GetUsers {
    /// All users registered in the service.
    fn users(&self) -> &RwLock<HashMap<AccountId, Mutex<WriteCommands>>>;
}

/// Use with db_write macro.
pub trait WriteDatabase {
    fn write_database_with_db_macro_do_not_call_this_outside_macros(
        &self,
    ) -> &RwLock<HashMap<AccountId, Mutex<WriteCommands>>>;
}

pub trait ReadDatabase {
    fn read_database(&self) -> ReadCommands;
}

pub trait GetCoreServerInternalApi {
    fn core_server_internal_api(&self) -> CoreServerInternalApi;
}

pub trait GetMediaServerInternalApi {
    fn media_server_internal_api(&self) -> MediaServerInternalApi;
}

/// Helper macro for getting write access to database.
///
/// Might make return error StatusCode::INTERNAL_SERVER_ERROR
/// if user ID does not exist.
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
            .get($user_id)
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR) // User does not exists
            .map(|x| async { x.lock().await })
    };
}

pub(crate) use db_write;
