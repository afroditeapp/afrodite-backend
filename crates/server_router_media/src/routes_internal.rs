//! Routes for server to server connections

use axum::{routing::post, Router};
use server_common::app::GetConfig;
use server_state::S;

use crate::api;

// TODO: Use TLS for checking that all internal communication comes from trusted
//       sources.

/// Internal route handlers for server to server communication.
pub struct InternalApp;

impl InternalApp {
    pub fn create_media_server_router(state: S) -> Router {
        let mut router = Router::new();

        if state.config().internal_api_config().microservice {
            router = router.route(
                api::media_internal::PATH_INTERNAL_GET_CHECK_MODERATION_REQUEST_FOR_ACCOUNT,
                post(api::media_internal::internal_get_check_moderation_request_for_account::<S>),
            );
        }

        router.with_state(state)
    }
}
