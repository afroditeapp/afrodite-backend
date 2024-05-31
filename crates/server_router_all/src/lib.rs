#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]



use axum::{
    routing::{get},
    Router,
};
use simple_backend::{web_socket::WebSocketManager};

use server_state::S;

mod api;

pub fn create_connect_router(
    state: S,
    ws_manager: WebSocketManager,
) -> Router {
    Router::new()
        .route(
            api::common::PATH_CONNECT, // This route checks the access token by itself.
            get({
                move |state, param1, param2, param3| {
                    api::connection::get_connect_websocket::<S>(
                        state, param1, param2, param3, ws_manager,
                    )
                }
            }),
        )
        .with_state(state)
}
