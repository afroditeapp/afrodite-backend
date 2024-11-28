use axum::{middleware, Router};
use server_state::S;

use crate::api;

/// Private routes only accessible when WebSocket is connected.
pub struct ConnectedApp {
    state: S,
}

impl ConnectedApp {
    pub fn new(state: S) -> Self {
        Self { state }
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
                    api::utils::authenticate_with_access_token,
                )
            })
    }
}
