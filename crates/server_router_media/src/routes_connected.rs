use axum::{middleware, Router};
use server_state::StateForRouterCreation;

use crate::api;

/// Private routes only accessible when WebSocket is connected.
pub struct ConnectedApp {
    state: StateForRouterCreation,
}

impl ConnectedApp {
    pub fn new(state: StateForRouterCreation) -> Self {
        Self {
            state,
        }
    }

    pub fn private_media_server_router(&self) -> Router {
        Router::new()
            // Media
            .merge(api::media::router_media_content(self.state.clone()))
            .merge(api::media::router_profile_content(self.state.clone()))
            .merge(api::media::router_security_content(self.state.clone()))
            .merge(api::media::router_content(self.state.clone()))
            .merge(api::media::router_tile_map(self.state.clone()))
            // Media admin
            .merge(api::media_admin::router_admin_moderation(
                self.state.clone(),
            ))
            .route_layer({
                middleware::from_fn_with_state(
                    self.state.s.clone(),
                    api::utils::authenticate_with_access_token,
                )
            })
    }
}
