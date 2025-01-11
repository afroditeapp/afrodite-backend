use axum::{middleware, Router};
use server_api::app::GetConfig;
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

    pub fn state(&self) -> S {
        self.state.clone()
    }

    pub fn private_profile_server_router(&self) -> Router {
        let private = Router::new()
            .merge(api::profile::router_filters(self.state.clone()))
            .merge(api::profile::router_profile_data(self.state.clone()))
            .merge(api::profile::router_location(self.state.clone()))
            .merge(api::profile::router_favorite(self.state.clone()))
            .merge(api::profile::router_iterate_profiles(self.state.clone()))
            .merge(api::profile::router_statistics(self.state.clone()))
            .merge(api::profile_admin::router_admin_statistics(
                self.state.clone(),
            ))
            .merge(api::profile_admin::router_admin_profile_data(
                self.state.clone(),
            ))
            .merge(api::profile_admin::router_admin_profile_name_allowlist(
                self.state.clone(),
            ))
            .merge(api::profile_admin::router_admin_profile_text(
                self.state.clone(),
            ));

        let private = if self.state.config().debug_mode() {
            private.merge(api::profile::router_benchmark(self.state.clone()))
        } else {
            private
        };

        private.route_layer({
            middleware::from_fn_with_state(self.state(), api::utils::authenticate_with_access_token)
        })
    }
}
