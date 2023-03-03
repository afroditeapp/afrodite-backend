//! HTTP API types and request handlers for all servers.

pub mod core;
pub mod media;

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

use self::core::{
    user::{ApiKey, UserId},
    API_KEY_HEADER_STR,
};

// Paths

pub const PATH_PREFIX: &str = "/api/v1/";

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        core::register,
        core::login,
        core::get_profile,
        core::post_profile,
        core::internal::check_api_key,
        media::get_image,
        media::internal::post_image,
    ),
    components(schemas(
        core::user::UserId,
        core::user::ApiKey,
        core::profile::Profile,
        media::image::ImageFileName,
        media::image::ImageFile,
    )),
    modifiers(&SecurityApiTokenDefault),
)]
pub struct ApiDoc;

pub struct SecurityApiTokenDefault;

impl Modify for SecurityApiTokenDefault {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(utoipa::openapi::security::ApiKey::Header(
                    ApiKeyValue::new(API_KEY_HEADER_STR),
                )),
            )
        }
    }
}

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
    fn users(&self) -> &RwLock<HashMap<UserId, Mutex<WriteCommands>>>;
}

/// Use with db_write macro.
pub trait WriteDatabase {
    fn write_database_with_db_macro_do_not_call_this_outside_macros(
        &self,
    ) -> &RwLock<HashMap<UserId, Mutex<WriteCommands>>>;
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
