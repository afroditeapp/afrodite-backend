use axum::{
    middleware,
    routing::{get, patch, post, put, delete},
    Router,
};

use super::AppState;
use crate::api::{self};

/// Private routes only accessible when WebSocket is connected.
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
                    move |param1, param2, param3, param4| {
                        api::common_admin::post_request_update_software(
                            param1, param2, param3, param4, state,
                        )
                    }
                }),
            )
            .route(
                api::common_admin::PATH_POST_REQUEST_RESTART_OR_RESET_BACKEND,
                post({
                    let state = self.state.clone();
                    move |param1, param2| {
                        api::common_admin::post_request_restart_or_reset_backend(
                            param1, param2, state,
                        )
                    }
                }),
            )
            .route(
                api::common_admin::PATH_GET_BACKEND_CONFIG,
                get({
                    let state = self.state.clone();
                    move |param1| {
                        api::common_admin::get_backend_config(
                            param1, state,
                        )
                    }
                }),
            )
            .route(
                api::common_admin::PATH_POST_BACKEND_CONFIG,
                post({
                    let state = self.state.clone();
                    move |param1, param2| {
                        api::common_admin::post_backend_config(
                            param1, param2, state,
                        )
                    }
                }),
            )
            .route(
                api::common_admin::PATH_GET_PERF_DATA,
                get({
                    let state = self.state.clone();
                    move |param1, param2, param3| {
                        api::common_admin::get_perf_data(
                            param1, param2, param3, state,
                        )
                    }
                }),
            )
            .route_layer({
                middleware::from_fn({
                    let state = self.state.clone();
                    move |addr, req, next| {
                        api::utils::authenticate_with_access_token(state.clone(), addr, req, next)
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
                api::account::PATH_GET_ACCOUNT_SETUP,
                get({
                    let state = self.state.clone();
                    move |body| api::account::get_account_setup(body, state)
                }),
            )
            .route(
                api::account::PATH_GET_ACCOUNT_DATA,
                get({
                    let state = self.state.clone();
                    move |body| api::account::get_account_data(body, state)
                }),
            )
            .route(
                api::account::PATH_POST_ACCOUNT_SETUP,
                post({
                    let state = self.state.clone();
                    move |arg1, arg2| api::account::post_account_setup(arg1, arg2, state)
                }),
            )
            .route(
                api::account::PATH_POST_ACCOUNT_DATA,
                post({
                    let state = self.state.clone();
                    move |arg1, arg2| api::account::post_account_data(arg1, arg2, state)
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
                    api::utils::authenticate_with_access_token(state.clone(), addr, req, next)
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
                api::profile::PATH_GET_LOCATION,
                get({
                    let state = self.state.clone();
                    move |param1| api::profile::get_location(param1, state)
                }),
            )
            .route(
                api::profile::PATH_GET_FAVORITE_PROFILES,
                get({
                    let state = self.state.clone();
                    move |param1| api::profile::get_favorite_profiles(param1, state)
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
            .route(
                api::profile::PATH_POST_FAVORITE_PROFILE,
                post({
                    let state = self.state.clone();
                    move |param1, param2| api::profile::post_favorite_profile(param1, param2, state)
                }),
            )
            .route(
                api::profile::PATH_DELETE_FAVORITE_PROFILE,
                delete({
                    let state = self.state.clone();
                    move |param1, param2| api::profile::delete_favorite_profile(param1, param2, state)
                }),
            )
            .route_layer({
                middleware::from_fn({
                    let state = self.state.clone();
                    move |addr, req, next| {
                        api::utils::authenticate_with_access_token(state.clone(), addr, req, next)
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
                api::media::PATH_GET_MAP_TILE,
                get({
                    let state = self.state.clone();
                    move |param1, param2, param3| {
                        api::media::get_map_tile(param1, param2, param3, state)
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
                        api::utils::authenticate_with_access_token(state.clone(), addr, req, next)
                    }
                })
            });

        Router::new().merge(private)
    }

    pub fn private_chat_server_router(&self) -> Router {
        let private = Router::new()
            .route(
                api::chat::PATH_POST_SEND_LIKE,
                post({
                    let state = self.state.clone();
                    move |param1, param2| api::chat::post_send_like(param1, param2, state)
                }),
            )
            .route(
                api::chat::PATH_GET_SENT_LIKES,
                get({
                    let state = self.state.clone();
                    move |param1| api::chat::get_sent_likes(param1, state)
                }),
            )
            .route(
                api::chat::PATH_GET_RECEIVED_LIKES,
                get({
                    let state = self.state.clone();
                    move |param1| api::chat::get_received_likes(param1, state)
                }),
            )
            .route(
                api::chat::PATH_DELETE_LIKE,
                delete({
                    let state = self.state.clone();
                    move |param1, param2| api::chat::delete_like(param1, param2, state)
                }),
            )
            .route(
                api::chat::PATH_GET_MATCHES,
                get({
                    let state = self.state.clone();
                    move |param1| api::chat::get_matches(param1, state)
                }),
            )
            .route(
                api::chat::PATH_POST_BLOCK_PROFILE,
                post({
                    let state = self.state.clone();
                    move |param1, param2| api::chat::post_block_profile(param1, param2, state)
                }),
            )
            .route(
                api::chat::PATH_POST_UNBLOCK_PROFILE,
                post({
                    let state = self.state.clone();
                    move |param1, param2| api::chat::post_unblock_profile(param1, param2, state)
                }),
            )
            .route(
                api::chat::PATH_GET_SENT_BLOCKS,
                get({
                    let state = self.state.clone();
                    move |param1| api::chat::get_sent_blocks(param1, state)
                }),
            )
            .route(
                api::chat::PATH_GET_RECEIVED_BLOCKS,
                get({
                    let state = self.state.clone();
                    move |param1| api::chat::get_received_blocks(param1, state)
                }),
            )
            .route(
                api::chat::PATH_GET_PENDING_MESSAGES,
                get({
                    let state = self.state.clone();
                    move |param1| api::chat::get_pending_messages(param1, state)
                }),
            )
            .route(
                api::chat::PATH_DELETE_PENDING_MESSAGES,
                delete({
                    let state = self.state.clone();
                    move |param1, param2| api::chat::delete_pending_messages(param1, param2, state)
                }),
            )
            .route(
                api::chat::PATH_GET_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE,
                get({
                    let state = self.state.clone();
                    move |param1, param2| api::chat::get_message_number_of_latest_viewed_message(param1, param2, state)
                }),
            )
            .route(
                api::chat::PATH_POST_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE,
                post({
                    let state = self.state.clone();
                    move |param1, param2| api::chat::post_message_number_of_latest_viewed_message(param1, param2, state)
                }),
            )
            .route(
                api::chat::PATH_POST_SEND_MESSAGE,
                post({
                    let state = self.state.clone();
                    move |param1, param2| api::chat::post_send_message(param1, param2, state)
                }),
            )
            .route_layer({
                middleware::from_fn({
                    let state = self.state.clone();
                    move |addr, req, next| {
                        api::utils::authenticate_with_access_token(state.clone(), addr, req, next)
                    }
                })
            });

        Router::new().merge(private)
    }
}
