//! Routes for server to server connections

use axum::{routing::post, Router};
use server_api::app::GetConfig;
use server_state::S;

use crate::api;

// TODO: Use TLS for checking that all internal communication comes from trusted
//       sources.

/// Internal route handlers for server to server communication.
pub struct InternalApp;

impl InternalApp {
    pub fn create_account_server_router(state: S) -> Router {
        let mut router = Router::new();

        if state.config().internal_api_config().bot_login {
            router = router
                .route(
                    api::account_internal::PATH_REGISTER,
                    post(api::account_internal::post_register),
                )
                .route(
                    api::account_internal::PATH_LOGIN,
                    post(api::account_internal::post_login),
                )
        }

        router.with_state(state)
    }
}
