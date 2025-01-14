//! Routes for server to server connections

use axum::Router;
use server_state::S;

// TODO: Use TLS for checking that all internal communication comes from trusted
//       sources.

/// Internal route handlers for server to server communication.
pub struct InternalApp;

impl InternalApp {
    pub fn create_account_server_router(_state: S) -> Router {
        Router::new()
    }
}
