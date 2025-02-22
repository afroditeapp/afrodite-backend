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


    pub fn private_chat_server_router(&self) -> Router {
        Router::new()
            // Chat
            .merge(api::chat::like::router_like(self.state.clone()))
            .merge(api::chat::block::router_block(self.state.clone()))
            .merge(api::chat::match_routes::router_match(self.state.clone()))
            .merge(api::chat::message::router_message(self.state.clone()))
            .merge(api::chat::public_key::router_public_key(self.state.clone()))
            .merge(
                api::chat::push_notifications::router_push_notification_private(self.state.clone()),
            )
            .merge(
                api::chat::report::router_chat_report(self.state.clone()),
            )
            .route_layer({
                middleware::from_fn_with_state(
                    self.state.s.clone(),
                    api::utils::authenticate_with_access_token,
                )
            })
    }
}
