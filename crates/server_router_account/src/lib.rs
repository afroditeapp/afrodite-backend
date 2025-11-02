#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use axum::{
    Router,
    routing::{any, get, post},
};
use routes_connected::ConnectedApp;
use server_api::app::GetConfig;
use server_state::StateForRouterCreation;

mod api;
mod routes_bot;
mod routes_connected;

pub use routes_bot::{LocalBotApiRoutes, RemoteBotApiRoutes};
use simple_backend::web_socket::WebSocketManager;

pub struct CommonRoutes;

impl CommonRoutes {
    pub fn routes_without_obfuscation_support(
        state: StateForRouterCreation,
        ws_manager: WebSocketManager,
    ) -> Router {
        Router::new()
            .route(
                api::common::PATH_GET_VERSION,
                get(server_api::common::get_version),
            )
            .route(
                api::common::PATH_FILE_PACKAGE_ACCESS,
                get(server_api::common::get_file_package_access),
            )
            .route(
                api::common::PATH_FILE_PACKAGE_ACCESS_ROOT,
                get(server_api::common::get_file_package_access_root),
            )
            .route(
                api::common::PATH_FILE_PACKAGE_ACCESS_PWA_INDEX_HTML,
                get(server_api::common::get_file_package_access_pwa_index_html),
            )
            .route(
                api::common::PATH_CONNECT, // This route checks the access token by itself.
                // Use any to allow both GET and CONNECT methods.
                any({
                    move |state, param1, param2, param3| {
                        api::common::get_connect_websocket(
                            state, param1, param2, param3, ws_manager,
                        )
                    }
                }),
            )
            .with_state(state.s.clone())
    }

    pub fn routes_with_obfuscation_support(state: StateForRouterCreation) -> Router {
        let public = Router::new();
        public.merge(ConnectedApp::new(state).private_common_router())
    }
}

pub struct AccountRoutes;

impl AccountRoutes {
    pub fn routes_without_obfuscation_support(state: StateForRouterCreation) -> Router {
        Router::new()
            .route(
                api::account::PATH_SIGN_IN_WITH_LOGIN,
                post(api::account::post_sign_in_with_login),
            )
            .route(
                api::account::PATH_SIGN_IN_WITH_APPLE_REDIRECT_TO_APP,
                post(api::account::post_sign_in_with_apple_redirect_to_app),
            )
            .route(
                api::account::PATH_GET_VERIFY_EMAIL,
                get(api::account::get_verify_email),
            )
            .route(
                api::account::PATH_GET_VERIFY_NEW_EMAIL,
                get(api::account::get_verify_new_email),
            )
            .with_state(state.s.clone())
    }

    pub fn routes_with_obfuscation_support(state: StateForRouterCreation) -> Router {
        let public = Router::new();
        let public = if state.s.config().demo_account_config().is_some() {
            public.merge(api::account::router_demo(state.clone()))
        } else {
            public
        };
        public.merge(ConnectedApp::new(state).private_account_server_router())
    }
}
