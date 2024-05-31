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
