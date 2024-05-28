use axum::{middleware, Router};
use server_api as api;

use crate::app::S;

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
        Router::new()
            .merge(api::common_admin::manager_router(self.state.clone()))
            .merge(api::common_admin::config_router(self.state.clone()))
            .merge(api::common_admin::perf_router(self.state.clone()))
            .route_layer({
                middleware::from_fn_with_state(
                    self.state(),
                    api::utils::authenticate_with_access_token::<S>,
                )
            })
    }

    pub fn private_account_server_router(&self) -> Router {
        let private = Router::new()
            .merge(api::account::register_router(self.state.clone()))
            .merge(api::account::delete_router(self.state.clone()))
            .merge(api::account::settings_router(self.state.clone()))
            .merge(api::account::state_router(self.state.clone()));

        let private = if self.state.config.debug_mode() {
            private.merge(api::profile::benchmark_router(self.state.clone()))
        } else {
            private
        };

        private.route_layer({
            middleware::from_fn_with_state(
                self.state(),
                api::utils::authenticate_with_access_token::<S>,
            )
        })
    }

    pub fn private_profile_server_router(&self) -> Router {
        Router::new()
            .merge(api::profile::attributes_router(self.state.clone()))
            .merge(api::profile::profile_data_router(self.state.clone()))
            .merge(api::profile::location_router(self.state.clone()))
            .merge(api::profile::favorite_router(self.state.clone()))
            .merge(api::profile::iterate_profiles_router(self.state.clone()))
            .route_layer({
                middleware::from_fn_with_state(
                    self.state(),
                    api::utils::authenticate_with_access_token::<S>,
                )
            })
    }

    pub fn private_media_server_router(&self) -> Router {
        Router::new()
            // Media
            .merge(api::media::profile_content_router(self.state.clone()))
            .merge(api::media::security_content_router(self.state.clone()))
            .merge(api::media::moderation_request_router(self.state.clone()))
            .merge(api::media::content_router(self.state.clone()))
            .merge(api::media::tile_map_router(self.state.clone()))
            // Media admin
            .merge(api::media_admin::admin_moderation_router(
                self.state.clone(),
            ))
            .route_layer({
                middleware::from_fn_with_state(
                    self.state.clone(),
                    api::utils::authenticate_with_access_token::<S>,
                )
            })
    }

    pub fn private_chat_server_router(&self) -> Router {
        Router::new()
            // Chat
            .merge(api::chat::like::like_router(self.state.clone()))
            .merge(api::chat::block::block_router(self.state.clone()))
            .merge(api::chat::match_routes::match_router(self.state.clone()))
            .merge(api::chat::message::message_router(self.state.clone()))
            .merge(
                api::chat::push_notifications::push_notification_router_private(self.state.clone()),
            )
            .route_layer({
                middleware::from_fn_with_state(
                    self.state.clone(),
                    api::utils::authenticate_with_access_token::<S>,
                )
            })
    }
}
