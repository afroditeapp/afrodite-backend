#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use axum::Router;
use routes_connected::ConnectedApp;
use server_state::S;

mod api;
mod routes_connected;

pub fn create_chat_server_router(state: S) -> Router {
    let public = Router::new()
        .merge(api::chat::push_notifications::router_push_notification_public(state.clone()));

    public.merge(ConnectedApp::new(state).private_chat_server_router())
}
