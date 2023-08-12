use axum::{
    middleware,
    Router,
    routing::{get, patch, post, put},
};

use crate::api::{self};

use super::AppState;

/// Private routes only accessible when WebSocket is connected.
/// Debug mode allows also connection without the WebSocket connection.
pub struct ConnectedApp {
    state: AppState,
}

impl ConnectedApp {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn state(&self) -> AppState {
        self.state.clone()
    }

    pub fn private_common_router(&self) -> Router {
        let private = Router::new()
            .route(
                api::common_admin::PATH_GET_SYSTEM_INFO,
                get({
                    let state = self.state.clone();
                    move |param1| api::common_admin::get_system_info(param1, state)
                }),
            )
            .route(
                api::common_admin::PATH_GET_SOFTWARE_INFO,
                get({
                    let state = self.state.clone();
                    move |param1| api::common_admin::get_software_info(param1, state)
                }),
            )
            .route(
                api::common_admin::PATH_GET_LATEST_BUILD_INFO,
                get({
                    let state = self.state.clone();
                    move |param1, param2| {
                        api::common_admin::get_latest_build_info(param1, param2, state)
                    }
                }),
            )
            .route(
                api::common_admin::PATH_POST_REQUEST_BUILD_SOFTWARE,
                post({
                    let state = self.state.clone();
                    move |param1, param2| {
                        api::common_admin::post_request_build_software(param1, param2, state)
                    }
                }),
            )
            .route(
                api::common_admin::PATH_POST_REQUEST_UPDATE_SOFTWARE,
                post({
                    let state = self.state.clone();
                    move |param1, param2, param3| {
                        api::common_admin::post_request_update_software(
                            param1, param2, param3, state,
                        )
                    }
                }),
            )
            .route_layer({
                middleware::from_fn({
                    let state = self.state.clone();
                    move |addr, req, next| {
                        api::utils::authenticate_with_api_key(state.clone(), addr, req, next)
                    }
                })
            });

        Router::new().merge(private)
    }

    pub fn private_account_server_router(&self) -> Router {
        let private = Router::new()
            .route(
                api::account::PATH_ACCOUNT_STATE,
                get({
                    let state = self.state.clone();
                    move |body| api::account::get_account_state(body, state)
                }),
            )
            .route(
                api::account::PATH_ACCOUNT_SETUP,
                post({
                    let state = self.state.clone();
                    move |arg1, arg2| api::account::post_account_setup(arg1, arg2, state)
                }),
            )
            .route(
                api::account::PATH_ACCOUNT_COMPLETE_SETUP,
                post({
                    let state = self.state.clone();
                    move |arg1| api::account::post_complete_setup(arg1, state)
                }),
            )
            .route(
                api::account::PATH_SETTING_PROFILE_VISIBILITY,
                put({
                    let state = self.state.clone();
                    move |p1, p2| api::account::put_setting_profile_visiblity(p1, p2, state)
                }),
            );

        let private = if self.state.config.debug_mode() {
            private
                .route(
                    api::profile::PATH_GET_PROFILE_FROM_DATABASE_BENCHMARK,
                    get({
                        let state = self.state.clone();
                        move |param1, param2| {
                            api::profile::get_profile_from_database_debug_mode_benchmark(
                                param1, param2, state,
                            )
                        }
                    }),
                )
                .route(
                    api::profile::PATH_POST_PROFILE_TO_DATABASE_BENCHMARK,
                    post({
                        let state = self.state.clone();
                        move |param1, param2| {
                            api::profile::post_profile_to_database_debug_mode_benchmark(
                                param1, param2, state,
                            )
                        }
                    }),
                )
        } else {
            private
        };

        let private = private.route_layer({
            middleware::from_fn({
                let state = self.state.clone();
                move |addr, req, next| {
                    api::utils::authenticate_with_api_key(state.clone(), addr, req, next)
                }
            })
        });

        Router::new().merge(private)
    }

    pub fn private_profile_server_router(&self) -> Router {
        let private = Router::new()
            .route(
                api::profile::PATH_GET_PROFILE,
                get({
                    let state = self.state.clone();
                    move |param1, param2| api::profile::get_profile(param1, param2, state)
                }),
            )
            .route(
                api::profile::PATH_POST_PROFILE,
                post({
                    let state = self.state.clone();
                    move |header, body| api::profile::post_profile(header, body, state)
                }),
            )
            .route(
                api::profile::PATH_PUT_LOCATION,
                put({
                    let state = self.state.clone();
                    move |p1, p2| api::profile::put_location(p1, p2, state)
                }),
            )
            .route(
                api::profile::PATH_POST_NEXT_PROFILE_PAGE,
                post({
                    let state = self.state.clone();
                    move |p1| api::profile::post_get_next_profile_page(p1, state)
                }),
            )
            .route(
                api::profile::PATH_POST_RESET_PROFILE_PAGING,
                post({
                    let state = self.state.clone();
                    move |p1| api::profile::post_reset_profile_paging(p1, state)
                }),
            )
            .route_layer({
                middleware::from_fn({
                    let state = self.state.clone();
                    move |addr, req, next| {
                        api::utils::authenticate_with_api_key(state.clone(), addr, req, next)
                    }
                })
            });

        Router::new().merge(private)
    }

    pub fn private_media_server_router(&self) -> Router {
        let private = Router::new()
            .route(
                api::media::PATH_GET_IMAGE,
                get({
                    let state = self.state.clone();
                    move |param1, param2, param3| {
                        api::media::get_image(param1, param2, param3, state)
                    }
                }),
            )
            .route(
                api::media::PATH_GET_PRIMARY_IMAGE_INFO,
                get({
                    let state = self.state.clone();
                    move |param1, param2, param3| {
                        api::media::get_primary_image_info(param1, param2, param3, state)
                    }
                }),
            )
            .route(
                api::media_admin::PATH_GET_SECURITY_IMAGE_INFO,
                get({
                    let state = self.state.clone();
                    move |param1, param2| {
                        api::media_admin::get_security_image_info(param1, param2, state)
                    }
                }),
            )
            .route(
                api::media::PATH_GET_ALL_NORMAL_IMAGES_INFO,
                get({
                    let state = self.state.clone();
                    move |param1, param2| api::media::get_all_normal_images(param1, param2, state)
                }),
            )
            .route(
                api::media::PATH_PUT_PRIMARY_IMAGE,
                put({
                    let state = self.state.clone();
                    move |param1, param2| api::media::put_primary_image(param1, param2, state)
                }),
            )
            .route(
                api::media::PATH_MODERATION_REQUEST,
                get({
                    let state = self.state.clone();
                    move |param1| api::media::get_moderation_request(param1, state)
                }),
            )
            .route(
                api::media::PATH_MODERATION_REQUEST,
                put({
                    let state = self.state.clone();
                    move |param1, param2| api::media::put_moderation_request(param1, param2, state)
                }),
            )
            .route(
                api::media::PATH_MODERATION_REQUEST_SLOT,
                put({
                    let state = self.state.clone();
                    move |param1, param2, param3| {
                        api::media::put_image_to_moderation_slot(param1, param2, param3, state)
                    }
                }),
            )
            .route(
                api::media_admin::PATH_ADMIN_MODERATION_PAGE_NEXT,
                patch({
                    let state = self.state.clone();
                    move |param1| api::media_admin::patch_moderation_request_list(param1, state)
                }),
            )
            .route(
                api::media_admin::PATH_ADMIN_MODERATION_HANDLE_REQUEST,
                post({
                    let state = self.state.clone();
                    move |param1, param2, param3| {
                        api::media_admin::post_handle_moderation_request(
                            param1, param2, param3, state,
                        )
                    }
                }),
            )
            .route_layer({
                middleware::from_fn({
                    let state = self.state.clone();
                    move |addr, req, next| {
                        api::utils::authenticate_with_api_key(state.clone(), addr, req, next)
                    }
                })
            });

        Router::new().merge(private)
    }

    pub fn private_chat_server_router(&self) -> Router {
        let private = Router::new()
            // .route(
            //     api::media::PATH_GET_IMAGE,
            //     get({
            //         let state = self.state.clone();
            //         move |param1, param2, param3| api::media::get_image(param1, param2, param3, state)
            //     }),
            // )
          ;

        Router::new().merge(private)
    }
}
