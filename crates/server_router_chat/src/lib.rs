#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use axum::Router;
use routes_connected::ConnectedApp;
use server_state::StateForRouterCreation;

mod api;
mod routes_connected;

pub fn create_chat_server_router(state: StateForRouterCreation) -> Router {
    let public = Router::new();

    public.merge(ConnectedApp::new(state).private_chat_server_router())
}
