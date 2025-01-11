use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use super::AppState;
use crate::api::{self};

/// Private routes only accessible with correct API key.
pub struct PrivateRoutes {
    state: AppState,
}

impl PrivateRoutes {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub fn state(&self) -> AppState {
        self.state.clone()
    }

    pub fn private_manager_server_router(&self) -> Router {
        let private = Router::new()
            .route(
                api::manager::PATH_GET_ENCRYPTION_KEY,
                get({
                    let state = self.state.clone();
                    move |param1, param2| api::manager::get_encryption_key(param1, param2, state)
                }),
            )
            .route(
                api::manager::PATH_GET_LATEST_SOFTWARE,
                get({
                    let state = self.state.clone();
                    move |param1, param2, param3| {
                        api::manager::get_latest_software(param1, param2, param3, state)
                    }
                }),
            )
            .route(
                api::manager::PATH_POST_RQUEST_SOFTWARE_UPDATE,
                post({
                    let state = self.state.clone();
                    move |param1, param2, param3, param4| {
                        api::manager::post_request_software_update(
                            param1, param2, param3, param4, state,
                        )
                    }
                }),
            )
            .route(
                api::manager::PATH_POST_RQUEST_RESTART_OR_RESET_BACKEND,
                post({
                    let state = self.state.clone();
                    move |param1, param2| {
                        api::manager::post_request_restart_or_reset_backend(param1, param2, state)
                    }
                }),
            )
            .route(
                api::manager::PATH_GET_SOFTWARE_INFO,
                get({
                    let state = self.state.clone();
                    move |param1| api::manager::get_software_info(param1, state)
                }),
            )
            .route(
                api::manager::PATH_GET_SYSTEM_INFO,
                get({
                    let state = self.state.clone();
                    move |param1| api::manager::get_system_info(param1, state)
                }),
            )
            .route(
                api::manager::PATH_GET_SYSTEM_INFO_ALL,
                get({
                    let state = self.state.clone();
                    move |param1| api::manager::get_system_info_all(param1, state)
                }),
            )
            .route_layer({
                middleware::from_fn({
                    let state = self.state.clone();
                    move |addr, req, next| {
                        api::utils::authenticate_with_api_key(state.clone(), addr, req, next)
                    }
                })
            });

        Router::new().merge(private)
    }
}
