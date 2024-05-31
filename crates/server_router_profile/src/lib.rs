#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use axum::Router;
use routes_connected::ConnectedApp;
use server_state::S;

mod api;
mod routes_connected;

pub fn create_profile_server_router(state: S) -> Router {
    let public = Router::new();

    public.merge(ConnectedApp::new(state).private_profile_server_router())
}
