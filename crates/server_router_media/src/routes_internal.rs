//! Routes for server to server connections

use axum::Router;
use server_state::S;

/// Internal route handlers for server to server communication.
pub struct InternalApp;

impl InternalApp {
    pub fn create_media_server_router(state: S) -> Router {
        Router::new().with_state(state)
    }
}
