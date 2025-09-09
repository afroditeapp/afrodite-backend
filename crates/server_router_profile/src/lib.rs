#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use axum::Router;
use routes_connected::ConnectedApp;
use server_state::StateForRouterCreation;

mod api;
mod routes_connected;

pub struct ProfileRoutes;

impl ProfileRoutes {
    pub fn routes_without_obfuscation_support(_state: StateForRouterCreation) -> Router {
        Router::new()
    }

    pub fn routes_with_obfuscation_support(state: StateForRouterCreation) -> Router {
        let public = Router::new();
        public.merge(ConnectedApp::new(state).private_profile_server_router())
    }
}
