use axum::{Router, middleware};
use server_api::app::GetConfig;
use server_state::{S, StateForRouterCreation};

use crate::api;

/// Private routes only accessible when WebSocket is connected.
pub struct ConnectedApp {
    state: StateForRouterCreation,
}

impl ConnectedApp {
    pub fn new(state: StateForRouterCreation) -> Self {
        Self { state }
    }

    pub fn state(&self) -> S {
        self.state.s.clone()
    }

    pub fn private_profile_server_router(&self) -> Router {
        let private = Router::new()
            .merge(api::profile::router_filters(self.state.clone()))
            .merge(api::profile::router_profile_data(self.state.clone()))
            .merge(api::profile::router_profile_report(self.state.clone()))
            .merge(api::profile::router_location(self.state.clone()))
            .merge(api::profile::router_favorite(self.state.clone()))
            .merge(api::profile::router_iterate_profiles(self.state.clone()))
            .merge(api::profile::router_statistics(self.state.clone()))
            .merge(api::profile::router_notification(self.state.clone()))
            .merge(api::profile_admin::router_admin_statistics(
                self.state.clone(),
            ))
            .merge(api::profile_admin::router_admin_iterate_profiles(
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

        let private = if self.state.s.config().debug_mode() {
            private.merge(api::profile::router_benchmark(self.state.clone()))
        } else {
            private
        };

        private.route_layer({
            middleware::from_fn_with_state(self.state(), api::utils::authenticate_with_access_token)
        })
    }
}
