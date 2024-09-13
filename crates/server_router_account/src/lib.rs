#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use axum::{
    routing::{get, post},
    Router,
};
use routes_connected::ConnectedApp;
use server_api::app::GetConfig;
use server_state::S;

mod api;
mod routes_connected;
mod routes_internal;

pub use routes_internal::InternalApp;
use simple_backend::web_socket::WebSocketManager;

pub fn create_common_server_router(state: S, ws_manager: WebSocketManager) -> Router {
    let public = Router::new()
        .route(
            api::common::PATH_GET_VERSION, // TODO(prod): Make private?
            get(server_api::common::get_version::<S>),
        )
        .route(
            api::common::PATH_FILE_PACKAGE_ACCESS,
            get(server_api::common::get_file_package_access::<S>),
        )
        .route(
            api::common::PATH_FILE_PACKAGE_ACCESS_ROOT,
            get(server_api::common::get_file_package_access_root::<S>),
        )
        .route(
            api::common::PATH_CONNECT, // This route checks the access token by itself.
            get({
                move |state, param1, param2, param3| {
                    api::common::get_connect_websocket::<S>(
                        state, param1, param2, param3, ws_manager,
                    )
                }
            }),
        )
        .with_state(state.clone());

    public.merge(ConnectedApp::new(state).private_common_router())
}

pub fn create_account_server_router(state: S) -> Router {
    let public = Router::new()
        .route(
            api::account::PATH_SIGN_IN_WITH_LOGIN,
            post(api::account::post_sign_in_with_login::<S>),
        )
        .with_state(state.clone());

    let public = if state.config().demo_mode_config().is_some() {
        public.merge(api::account::demo_mode_router(state.clone()))
    } else {
        public
    };

    public.merge(ConnectedApp::new(state).private_account_server_router())
}
