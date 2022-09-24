//! HTTP API types for all servers.

pub mod core;
pub mod media;

use std::collections::HashMap;

use tokio::sync::{RwLock, Mutex};

use crate::server::{session::{SessionManager, UserState}, database::{RouterDatabaseHandle, write::WriteCommands}};

use self::core::user::{ApiKey, UserId};

// Paths

pub const PATH_PREFIX: &str = "/api/v1/";

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
