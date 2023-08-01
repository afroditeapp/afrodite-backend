pub mod connected_routes;
pub mod connection;
pub mod sign_in_with;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Json, Router,
};


use utoipa::OpenApi;

use crate::{
    api::{
        self, ApiDoc, GetApiKeys, GetConfig, GetInternalApi, GetUsers, ReadDatabase, SignInWith,
        WriteDatabase, GetManagerApi,
    },
    config::Config, media_backup::MediaBackupHandle,
};

use self::{
    connected_routes::ConnectedApp, connection::WebSocketManager, sign_in_with::SignInWithManager,
};

use super::{
    data::{
        commands::WriteCommandRunnerHandle,
        read::ReadCommands,
        utils::{AccountIdManager, ApiKeyManager},
        RouterDatabaseReadHandle,
    },
    internal::{InternalApiClient, InternalApiManager}, manager_client::{ManagerApiClient, ManagerApiManager, ManagerClientError},
};

use error_stack::{Result, ResultExt};

#[derive(Clone)]
pub struct AppState {
    database: Arc<RouterDatabaseReadHandle>,
    internal_api: Arc<InternalApiClient>,
    manager_api: Arc<ManagerApiClient>,
    config: Arc<Config>,
    sign_in_with: Arc<SignInWithManager>,
}

impl GetApiKeys for AppState {
    fn api_keys(&self) -> ApiKeyManager<'_> {
        self.database.api_key_manager()
    }
}

impl GetUsers for AppState {
    fn users(&self) -> AccountIdManager<'_> {
        self.database.account_id_manager()
    }
}

impl ReadDatabase for AppState {
    fn read_database(&self) -> ReadCommands<'_> {
        self.database.read()
    }
}

impl WriteDatabase for AppState {
    fn write_database(&self) -> &WriteCommandRunnerHandle {
        self.database.write()
    }
}

impl SignInWith for AppState {
    fn sign_in_with_manager(&self) -> &SignInWithManager {
        &self.sign_in_with
    }
}

impl GetInternalApi for AppState {
    fn internal_api(&self) -> InternalApiManager {
        InternalApiManager::new(
            &self.config,
            &self.internal_api,
            self.api_keys(),
            self.read_database(),
            self.write_database(),
            self.database.account_id_manager(),
        )
    }
}

impl GetManagerApi for AppState {
    fn manager_api(&self) -> ManagerApiManager {
        ManagerApiManager::new(
            &self.config,
            &self.manager_api,
        )
    }
}

impl GetConfig for AppState {
    fn config(&self) -> &Config {
        &self.config
    }
}

pub struct App {
    state: AppState,
    ws_manager: Option<WebSocketManager>,
}

impl App {
    pub async fn new(
        database_handle: RouterDatabaseReadHandle,
        config: Arc<Config>,
        ws_manager: WebSocketManager,
    ) -> Result<Self, ManagerClientError> {
        let state = AppState {
            config: config.clone(),
            database: Arc::new(database_handle),
            internal_api: InternalApiClient::new(config.external_service_urls().clone()).into(),
            manager_api: ManagerApiClient::new(&config)?.into(),
            sign_in_with: SignInWithManager::new(config).into(),
        };

        Ok(Self {
            state,
            ws_manager: Some(ws_manager),
        })
    }

    pub fn state(&self) -> AppState {
        self.state.clone()
    }

    pub fn create_common_server_router(&mut self) -> Router {
        let public = Router::new().route(
            api::common::PATH_CONNECT, // This route checks the access token by itself.
            get({
                let state = self.state.clone();
                let ws_manager = self.ws_manager.take().unwrap(); // Only one instance required.
                move |param1, param2, param3| {
                    api::common::get_connect_websocket(param1, param2, param3, state, ws_manager)
                }
            }),
        )
        .route(
            api::common::PATH_GET_VERSION,
            get({
                let state = self.state.clone();
                move || {
                    api::common::get_version(state)
                }
            }),
        );

        public.merge(ConnectedApp::new(self.state.clone()).private_common_router())
    }

    pub fn create_account_server_router(&self) -> Router {
        let public = Router::new()
            .route(
                api::account::PATH_SIGN_IN_WITH_LOGIN,
                post({
                    let state = self.state.clone();
                    move |body| api::account::post_sign_in_with_login(body, state)
                }),
            );

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
        let public = Router::new();

        public.merge(ConnectedApp::new(self.state.clone()).private_chat_server_router())
    }
}
