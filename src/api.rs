//! HTTP API types for all servers.

pub mod core;
pub mod media;

use crate::server::session::SessionManager;

// Paths

pub const PATH_PREFIX: &str = "/api/v1/";

// App state getters

pub trait GetSessionManager {
    fn session_manager(&self) -> &SessionManager;
}
