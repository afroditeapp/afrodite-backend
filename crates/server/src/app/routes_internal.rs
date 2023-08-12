//! Routes for server to server connections

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    api::{self, GetConfig},
    app::AppState,
};

// TODO: Use TLS for checking that all internal communication comes from trusted
//       sources.

/// Internal route handlers for server to server communication.
pub struct InternalApp;

impl InternalApp {
    pub fn create_account_server_router(state: AppState) -> Router {
        let mut router = Router::new()
            .route(
                api::account_internal::PATH_INTERNAL_CHECK_API_KEY,
                get({
                    let state = state.clone();
                    move |body| api::account_internal::check_api_key(body, state)
                }),
            )
            .route(
                api::account_internal::PATH_INTERNAL_GET_ACCOUNT_STATE,
                get({
                    let state = state.clone();
                    move |param1| api::account_internal::internal_get_account_state(param1, state)
                }),
            );

        if state.config().internal_api_config().bot_login {
            router = router
                .route(
                    api::account::PATH_REGISTER,
                    post({
                        let state = state.clone();
                        move || api::account::post_register(state)
                    }),
                )
                .route(
                    api::account::PATH_LOGIN,
                    post({
                        let state = state.clone();
                        move |body| api::account::post_login(body, state)
                    }),
                )
        }

        router
    }

    pub fn create_profile_server_router(state: AppState) -> Router {
        Router::new().route(
            api::profile_internal::PATH_INTERNAL_POST_UPDATE_PROFILE_VISIBLITY,
            post({
                let state = state.clone();
                move |p1, p2| {
                    api::profile_internal::internal_post_update_profile_visibility(p1, p2, state)
                }
            }),
        )
    }

    pub fn create_media_server_router(state: AppState) -> Router {
        Router::new()
            .route(
                api::media_internal::PATH_INTERNAL_GET_CHECK_MODERATION_REQUEST_FOR_ACCOUNT,
                post({
                    let state = state.clone();
                    move |parameter1| {
                        api::media_internal::internal_get_check_moderation_request_for_account(
                            parameter1, state,
                        )
                    }
                }),
            )
            .route(
                api::media_internal::PATH_INTERNAL_POST_UPDATE_PROFILE_IMAGE_VISIBLITY,
                post({
                    let state = state.clone();
                    move |p1, p2, p3| {
                        api::media_internal::internal_post_update_profile_image_visibility(
                            p1, p2, p3, state,
                        )
                    }
                }),
            )
    }

    pub fn create_chat_server_router(_state: AppState) -> Router {
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
