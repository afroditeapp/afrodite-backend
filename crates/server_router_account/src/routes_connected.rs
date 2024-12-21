use axum::{middleware, Router};
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

    pub fn private_common_router(&self) -> Router {
        Router::new()
            .merge(api::common_admin::manager_router(self.state.clone()))
            .merge(api::common_admin::config_router(self.state.clone()))
            .merge(api::common_admin::perf_router(self.state.clone()))
            .route_layer({
                middleware::from_fn_with_state(
                    self.state.clone(),
                    api::utils::authenticate_with_access_token,
                )
            })
    }

    pub fn private_account_server_router(&self) -> Router {
        let private = Router::new()
            .merge(api::account::register_router(self.state.clone()))
            .merge(api::account::logout_router(self.state.clone()))
            .merge(api::account::delete_router(self.state.clone()))
            .merge(api::account::settings_router(self.state.clone()))
            .merge(api::account::state_router(self.state.clone()))
            .merge(api::account::news_router(self.state.clone()))
            .merge(api::account_admin::admin_delete_router(self.state.clone()))
            .merge(api::account_admin::admin_news_router(self.state.clone()));

        private.route_layer({
            middleware::from_fn_with_state(
                self.state.clone(),
                api::utils::authenticate_with_access_token,
            )
        })
    }
}
