use axum::{Router, middleware};
use server_state::StateForRouterCreation;

use crate::api;

/// Private routes only accessible when WebSocket is connected.
pub struct ConnectedApp {
    state: StateForRouterCreation,
}

impl ConnectedApp {
    pub fn new(state: StateForRouterCreation) -> Self {
        Self { state }
    }

    pub fn private_common_router(&self) -> Router {
        Router::new()
            .merge(api::common::router_client_config(self.state.clone()))
            .merge(api::common::router_data_export(self.state.clone()))
            .merge(api::common::router_push_notification_private(
                self.state.clone(),
            ))
            .merge(api::common_admin::router_maintenance(self.state.clone()))
            .merge(api::common_admin::router_manager(self.state.clone()))
            .merge(api::common_admin::router_config(self.state.clone()))
            .merge(api::common_admin::router_statistics(self.state.clone()))
            .merge(api::common_admin::router_report(self.state.clone()))
            .merge(api::common_admin::router_notification(self.state.clone()))
            .route_layer({
                middleware::from_fn_with_state(
                    self.state.clone(),
                    api::utils::authenticate_with_access_token,
                )
            })
    }

    pub fn private_account_server_router(&self) -> Router {
        let private = Router::new()
            .merge(api::account::router_register(self.state.clone()))
            .merge(api::account::router_logout(self.state.clone()))
            .merge(api::account::router_ban(self.state.clone()))
            .merge(api::account::router_delete(self.state.clone()))
            .merge(api::account::router_settings(self.state.clone()))
            .merge(api::account::router_state(self.state.clone()))
            .merge(api::account::router_news(self.state.clone()))
            .merge(api::account::router_account_report(self.state.clone()))
            .merge(api::account::router_client_features(self.state.clone()))
            .merge(api::account::router_notification(self.state.clone()))
            .merge(api::account_admin::router_admin_ban(self.state.clone()))
            .merge(api::account_admin::router_admin_delete(self.state.clone()))
            .merge(api::account_admin::router_admin_news(self.state.clone()))
            .merge(api::account_admin::router_admin_search(self.state.clone()))
            .merge(api::account_admin::router_admin_permissions(
                self.state.clone(),
            ))
            .merge(api::account_admin::router_admin_state(self.state.clone()))
            .merge(api::account_admin::router_admin_client_version(
                self.state.clone(),
            ));

        private.route_layer({
            middleware::from_fn_with_state(
                self.state.clone(),
                api::utils::authenticate_with_access_token,
            )
        })
    }
}
