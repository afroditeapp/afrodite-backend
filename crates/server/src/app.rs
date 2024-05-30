use std::{sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use config::{Config};
pub use server_api::app::*;
use server_api::{internal_api::InternalApiClient};
use server_common::push_notifications::{
    PushNotificationSender,
};
use server_data::{
    content_processing::ContentProcessingManagerData, db_manager::{RouterDatabaseReadHandle}, write_commands::{WriteCommandRunnerHandle}
};
use server_data_all::demo::DemoModeManager;
use simple_backend::{
    app::{
        SimpleBackendAppState,
    },
    web_socket::WebSocketManager,
};

use self::routes_connected::ConnectedApp;

use crate::api;

pub mod state_impl;
pub mod routes_connected;
pub mod routes_internal;

/// State type for route handlers.
pub type S = AppState;

#[derive(Clone)]
pub struct AppState {
    database: Arc<RouterDatabaseReadHandle>,
    write_queue: Arc<WriteCommandRunnerHandle>,
    internal_api: Arc<InternalApiClient>,
    config: Arc<Config>,
    content_processing: Arc<ContentProcessingManagerData>,
    demo_mode: DemoModeManager,
    push_notification_sender: PushNotificationSender,
    simple_backend_state: SimpleBackendAppState,
}

pub struct App {
    state: S,
    web_socket_manager: Option<WebSocketManager>,
}

impl App {
    pub async fn create_app_state(
        database_handle: RouterDatabaseReadHandle,
        write_queue: WriteCommandRunnerHandle,
        config: Arc<Config>,
        content_processing: Arc<ContentProcessingManagerData>,
        demo_mode: DemoModeManager,
        push_notification_sender: PushNotificationSender,
        simple_backend_state: SimpleBackendAppState,
    ) -> AppState {
        let database = Arc::new(database_handle);
        let state = AppState {
            config: config.clone(),
            database: database.clone(),
            write_queue: Arc::new(write_queue),
            internal_api: InternalApiClient::new(config.external_service_urls().clone()).into(),
            content_processing,
            demo_mode,
            push_notification_sender,
            simple_backend_state,
        };

        state
    }

    pub fn new(state: S, web_socket_manager: WebSocketManager) -> Self {
        Self {
            state,
            web_socket_manager: web_socket_manager.into(),
        }
    }

    pub fn state(&self) -> S {
        self.state.clone()
    }

    pub fn create_common_server_router(&mut self) -> Router {
        let public = Router::new()
            .route(
                api::common::PATH_CONNECT, // This route checks the access token by itself.
                get({
                    let ws_manager = self
                        .web_socket_manager
                        .take()
                        .expect("This should be called only once");
                    move |state, param1, param2, param3| {
                        api::connection::get_connect_websocket::<S>(
                            state, param1, param2, param3, ws_manager,
                        )
                    }
                }),
            )
            .route(
                api::common::PATH_GET_VERSION,
                get(server_api::common::get_version::<S>),
            )
            .with_state(self.state());

        public.merge(ConnectedApp::new(self.state.clone()).private_common_router())
    }

    pub fn create_account_server_router(&self) -> Router {
        let public = Router::new()
            .route(
                api::account::PATH_SIGN_IN_WITH_LOGIN,
                post(api::account::post_sign_in_with_login::<S>),
            )
            .with_state(self.state());

        let public = if self.state.config().demo_mode_config().is_some() {
            public.merge(api::account::demo_mode_router(self.state.clone()))
        } else {
            public
        };

        public.merge(ConnectedApp::new(self.state.clone()).private_account_server_router())
    }

    pub fn create_profile_server_router(&self) -> Router {
        let public = Router::new();

        public.merge(ConnectedApp::new(self.state.clone()).private_profile_server_router())
    }

    pub fn create_media_server_router(&self) -> Router {
        let public = Router::new();

        public.merge(ConnectedApp::new(self.state.clone()).private_media_server_router())
    }

    pub fn create_chat_server_router(&self) -> Router {
        let public = Router::new().merge(
            api::chat::push_notifications::push_notification_router_public(
                self.state.clone(),
            ),
        );

        public.merge(ConnectedApp::new(self.state.clone()).private_chat_server_router())
    }
}
