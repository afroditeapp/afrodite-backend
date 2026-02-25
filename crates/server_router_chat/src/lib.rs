#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use axum::{Router, routing::any};
use private::PrivateRoutes;
use server_state::StateForRouterCreation;

mod api;
mod private;

pub struct ChatRoutes;

impl ChatRoutes {
    pub fn routes_without_obfuscation_support(state: StateForRouterCreation) -> Router {
        Router::new()
            .route(
                api::chat::transfer::PATH_BACKUP_TRANSFER, // This route checks the access token by itself.
                // Use any to allow both GET and CONNECT methods.
                any(api::chat::transfer::get_backup_transfer),
            )
            .with_state(state.s)
    }

    pub fn routes_with_obfuscation_support(state: StateForRouterCreation) -> Router {
        let public = Router::new();
        public.merge(PrivateRoutes::new(state).private_chat_server_router())
    }
}
