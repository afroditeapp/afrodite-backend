//! Routes for server to server connections



use axum::{
    routing::{get, post}, Router,
};










use crate::{
    api::{
        self,
    },
};

use super::{
    app::AppState,
};

// TODO: Use TLS for checking that all internal communication comes from trusted
//       sources.

/// Internal route handlers for server to server communication.
pub struct InternalApp;

impl InternalApp {
    pub fn create_account_server_router(state: AppState) -> Router {
        Router::new().route(
            api::account::internal::PATH_CHECK_API_KEY,
            get({
                let state = state.clone();
                move |body| api::account::internal::check_api_key(body, state)
            }),
        )
    }

    pub fn create_profile_server_router(_state: AppState) -> Router {
        Router::new()
    }

    pub fn create_media_server_router(state: AppState) -> Router {
        Router::new().route(
            api::media::internal::PATH_POST_IMAGE,
            post({
                let state = state.clone();
                move |parameter1, parameter2, header1, header2, body| {
                    api::media::internal::post_image(
                        parameter1, parameter2, header1, header2, body, state,
                    )
                }
            }),
        )
    }
}
