//! Routes for server to server connections

use axum::{
    routing::{post},
    Router,
};
use simple_backend::app::SimpleBackendAppState;

use crate::{
    api::{self},
    app::AppState,
};

// TODO: Use TLS for checking that all internal communication comes from trusted
//       sources.

type S = SimpleBackendAppState<AppState>;

/// Internal route handlers for server to server communication.
pub struct InternalApp;

impl InternalApp {
    pub fn create_account_server_router(state: S) -> Router {
        let mut router = Router::new();

        if state
            .business_logic_state()
            .config
            .internal_api_config()
            .bot_login
        {
            router = router
                .route(
                    api::account_internal::PATH_REGISTER,
                    post(api::account_internal::post_register::<S>),
                )
                .route(
                    api::account_internal::PATH_LOGIN,
                    post(api::account_internal::post_login::<S>),
                )
        }

        router.with_state(state)
    }

    pub fn create_profile_server_router(state: S) -> Router {
        Router::new()
            .with_state(state)
    }

    pub fn create_media_server_router(state: S) -> Router {
        let mut router = Router::new();

        if state
            .business_logic_state()
            .config
            .internal_api_config()
            .microservice
        {
            router = router.route(
                api::media_internal::PATH_INTERNAL_GET_CHECK_MODERATION_REQUEST_FOR_ACCOUNT,
                post(api::media_internal::internal_get_check_moderation_request_for_account::<S>),
            );
        }

        router.with_state(state)
    }

    pub fn create_chat_server_router(_state: S) -> Router {
        Router::new()
        // .route(
        //     api::media::internal::PATH_INTERNAL_GET_CHECK_MODERATION_REQUEST_FOR_ACCOUNT,
        //     post({
        //         let state = state.clone();
        //         move |parameter1| {
        //             api::media::internal::internal_get_check_moderation_request_for_account(
        //                 parameter1, state,
        //             )
        //         }
        //     }),
        // )
    }
}
