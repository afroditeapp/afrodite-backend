//! Routes for server to server connections

use axum::Router;
use server_api::app::GetConfig;
use server_state::S;

// TODO: Use TLS for checking that all internal communication comes from trusted
//       sources.

/// Internal route handlers for server to server communication.
pub struct InternalApp;

impl InternalApp {
    pub fn create_media_server_router(state: S) -> Router {
        let router = Router::new();

        if state.config().internal_api_config().microservice {
            // No routes at the moment
        }

        router.with_state(state)
    }
}
