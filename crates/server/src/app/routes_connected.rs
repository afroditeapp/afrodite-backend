use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use crate::app::S;

use super::AppState;
use crate::api::{self};

/// Private routes only accessible when WebSocket is connected.
pub struct ConnectedApp {
    state: S,
}

impl ConnectedApp {
    pub fn new(state: S) -> Self {
        Self { state }
    }

    pub fn state(&self) -> S {
        self.state.clone()
    }

    pub fn private_common_router(&self) -> Router {
        let private = Router::new()
            .route(
                api::common_admin::PATH_GET_SYSTEM_INFO,
                get(api::common_admin::get_system_info::<S>),
            )
            .route(
                api::common_admin::PATH_GET_SOFTWARE_INFO,
                get(api::common_admin::get_software_info::<S>),
            )
            .route(
                api::common_admin::PATH_GET_LATEST_BUILD_INFO,
                get(api::common_admin::get_latest_build_info::<S>),
            )
            .route(
                api::common_admin::PATH_POST_REQUEST_BUILD_SOFTWARE,
                post(api::common_admin::post_request_build_software::<S>),
            )
            .route(
                api::common_admin::PATH_POST_REQUEST_UPDATE_SOFTWARE,
                post(api::common_admin::post_request_update_software::<S>),
            )
            .route(
                api::common_admin::PATH_POST_REQUEST_RESTART_OR_RESET_BACKEND,
                post(api::common_admin::post_request_restart_or_reset_backend::<S>),
            )
            .route(
                api::common_admin::PATH_GET_BACKEND_CONFIG,
                get(api::common_admin::get_backend_config::<S>),
            )
            .route(
                api::common_admin::PATH_POST_BACKEND_CONFIG,
                post(api::common_admin::post_backend_config::<S>),
            )
            .route(
                api::common_admin::PATH_GET_PERF_DATA,
                get(api::common_admin::get_perf_data::<S>),
            )
            .route_layer({
                middleware::from_fn_with_state(self.state(), api::utils::authenticate_with_access_token::<S, _>)
            })
            .with_state(self.state());

        private
    }

    pub fn private_account_server_router(&self) -> Router {
        let private = Router::new()
            .merge(api::account::register_router(self.state.clone()))
            .merge(api::account::delete_router(self.state.clone()))
            .merge(api::account::settings_router(self.state.clone()))
            .merge(api::account::state_router(self.state.clone()));

        let private = if self.state.business_logic_state().config.debug_mode() {
            let r = Router::new()
                .route(
                    api::profile::PATH_GET_PROFILE_FROM_DATABASE_BENCHMARK,
                    get(api::profile::get_profile_from_database_debug_mode_benchmark::<S>),
                )
                .route(
                    api::profile::PATH_POST_PROFILE_TO_DATABASE_BENCHMARK,
                    post(api::profile::post_profile_to_database_debug_mode_benchmark::<S>),
                )
                .with_state(self.state());
            private.merge(r)
        } else {
            private
        };

        let private = private.route_layer({
            middleware::from_fn_with_state(self.state(), api::utils::authenticate_with_access_token::<S, _>)
        });

        private
    }

    pub fn private_profile_server_router(&self) -> Router {
        let private = Router::new()
            .route(
                api::profile::PATH_GET_PROFILE,
                get(api::profile::get_profile::<S>),
            )
            .route(
                api::profile::PATH_GET_LOCATION,
                get(api::profile::get_location::<S>),
            )
            .route(
                api::profile::PATH_GET_FAVORITE_PROFILES,
                get(api::profile::get_favorite_profiles::<S>),
            )
            .route(
                api::profile::PATH_POST_PROFILE,
                post(api::profile::post_profile::<S>),
            )
            .route(
                api::profile::PATH_PUT_LOCATION,
                put(api::profile::put_location::<S>),
            )
            .route(
                api::profile::PATH_POST_NEXT_PROFILE_PAGE,
                post(api::profile::post_get_next_profile_page::<S>),
            )
            .route(
                api::profile::PATH_POST_RESET_PROFILE_PAGING,
                post(api::profile::post_reset_profile_paging::<S>),
            )
            .route(
                api::profile::PATH_POST_FAVORITE_PROFILE,
                post(api::profile::post_favorite_profile::<S>),
            )
            .route(
                api::profile::PATH_DELETE_FAVORITE_PROFILE,
                delete(api::profile::delete_favorite_profile::<S>),
            )
            .route_layer({
                middleware::from_fn_with_state(self.state(), api::utils::authenticate_with_access_token::<S, _>)
            })
            .with_state(self.state());

        private
    }

    pub fn private_media_server_router(&self) -> Router {
        let private = Router::new()
            // Media
            .merge(api::media::profile_content_router(self.state.clone()))
            .merge(api::media::security_image_router(self.state.clone()))
            .merge(api::media::moderation_request_router(self.state.clone()))
            .merge(api::media::content_router(self.state.clone()))
            .merge(api::media::tile_map_router(self.state.clone()))
            // Media admin
            .merge(api::media_admin::admin_moderation_router(self.state.clone()))
            .route_layer({
                middleware::from_fn_with_state(self.state.clone(), api::utils::authenticate_with_access_token::<S, _>)
            });

        private
    }

    pub fn private_chat_server_router(&self) -> Router {
        let private = Router::new()
            .route(
                api::chat::PATH_POST_SEND_LIKE,
                post(api::chat::post_send_like::<S>),
            )
            .route(
                api::chat::PATH_GET_SENT_LIKES,
                get(api::chat::get_sent_likes::<S>),
            )
            .route(
                api::chat::PATH_GET_RECEIVED_LIKES,
                get(api::chat::get_received_likes::<S>),
            )
            .route(
                api::chat::PATH_DELETE_LIKE,
                delete(api::chat::delete_like::<S>),
            )
            .route(
                api::chat::PATH_GET_MATCHES,
                get(api::chat::get_matches::<S>),
            )
            .route(
                api::chat::PATH_POST_BLOCK_PROFILE,
                post(api::chat::post_block_profile::<S>),
            )
            .route(
                api::chat::PATH_POST_UNBLOCK_PROFILE,
                post(api::chat::post_unblock_profile::<S>),
            )
            .route(
                api::chat::PATH_GET_SENT_BLOCKS,
                get(api::chat::get_sent_blocks::<S>),
            )
            .route(
                api::chat::PATH_GET_RECEIVED_BLOCKS,
                get(api::chat::get_received_blocks::<S>),
            )
            .route(
                api::chat::PATH_GET_PENDING_MESSAGES,
                get(api::chat::get_pending_messages::<S>),
            )
            .route(
                api::chat::PATH_DELETE_PENDING_MESSAGES,
                delete(api::chat::delete_pending_messages::<S>),
            )
            .route(
                api::chat::PATH_GET_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE,
                get(api::chat::get_message_number_of_latest_viewed_message::<S>),
            )
            .route(
                api::chat::PATH_POST_MESSAGE_NUMBER_OF_LATEST_VIEWED_MESSAGE,
                post(api::chat::post_message_number_of_latest_viewed_message::<S>),
            )
            .route(
                api::chat::PATH_POST_SEND_MESSAGE,
                post(api::chat::post_send_message::<S>),
            )
            .route_layer({
                middleware::from_fn_with_state(self.state.clone(), api::utils::authenticate_with_access_token::<S, _>)
            })
            .with_state(self.state());

        private
    }
}
