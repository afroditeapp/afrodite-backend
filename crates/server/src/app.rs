use std::{sync::Arc, fmt::Debug};

use api::BackendVersionProvider;
use axum::{
    routing::{get, post},
    Router,
};
use config::Config;
use error_stack::Result;
use futures::Future;
use model::{AccountId, BackendVersion};

use self::{
    connection::WebSocketManager, routes_connected::ConnectedApp, sign_in_with::SignInWithManager,
};
use super::{
    data::{
        read::ReadCommands,
        utils::{AccountIdManager, AccessTokenManager},
        write_commands::{WriteCmds, WriteCommandRunnerHandle},
        write_concurrent::ConcurrentWriteHandle,
        DataError, RouterDatabaseReadHandle, RouterDatabaseWriteHandle,
    },
    internal::{InternalApiClient, InternalApiManager},
    manager_client::{ManagerApiClient, ManagerApiManager, ManagerClientError},
};
use crate::api::{
    self, GetAccessTokens, GetConfig, GetInternalApi, GetManagerApi, GetAccounts, ReadData, SignInWith,
    WriteData,
};

pub mod connection;
pub mod routes_connected;
pub mod routes_internal;
pub mod sign_in_with;

#[derive(Clone)]
pub struct AppState {
    database: Arc<RouterDatabaseReadHandle>,
    write_queue: Arc<WriteCommandRunnerHandle>,
    internal_api: Arc<InternalApiClient>,
    manager_api: Arc<ManagerApiClient>,
    config: Arc<Config>,
    sign_in_with: Arc<SignInWithManager>,
}

impl BackendVersionProvider for AppState {
    fn backend_version(&self) -> BackendVersion {
        BackendVersion {
            backend_code_version: self.config.backend_code_version().to_string(),
            backend_version: self.config.backend_semver_version().to_string(),
            protocol_version: "1.0.0".to_string(),
        }
    }
}

impl GetAccessTokens for AppState {
    fn access_tokens(&self) -> AccessTokenManager<'_> {
        self.database.access_token_manager()
    }
}

impl GetAccounts for AppState {
    fn accounts(&self) -> AccountIdManager<'_> {
        self.database.account_id_manager()
    }
}

impl ReadData for AppState {
    fn read(&self) -> ReadCommands<'_> {
        self.database.read()
    }
}

#[async_trait::async_trait]
impl WriteData for AppState {
    async fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = Result<CmdResult, DataError>> + Send + 'static,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        cmd: GetCmd,
    ) -> Result<CmdResult, DataError> {
        self.write_queue.write(cmd).await
    }

    async fn write_concurrent<
        CmdResult: Send + 'static,
        Cmd: Future<Output = Result<CmdResult, DataError>> + Send + 'static,
        GetCmd: FnOnce(ConcurrentWriteHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountId,
        cmd: GetCmd,
    ) -> Result<CmdResult, DataError> {
        self.write_queue.concurrent_write(account, cmd).await
    }
}

impl SignInWith for AppState {
    fn sign_in_with_manager(&self) -> &SignInWithManager {
        &self.sign_in_with
    }
}

impl GetInternalApi for AppState {
    fn internal_api(&self) -> InternalApiManager<Self> {
        InternalApiManager::new(self, &self.internal_api)
    }
}

impl GetManagerApi for AppState {
    fn manager_api(&self) -> ManagerApiManager {
        ManagerApiManager::new(&self.manager_api)
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
        _database_write_handle: RouterDatabaseWriteHandle,
        write_queue: WriteCommandRunnerHandle,
        config: Arc<Config>,
        ws_manager: WebSocketManager,
    ) -> Result<Self, ManagerClientError> {
        let state = AppState {
            config: config.clone(),
            database: Arc::new(database_handle),
            write_queue: Arc::new(write_queue),
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
        let public = Router::new()
            .route(
                api::common::PATH_CONNECT, // This route checks the access token by itself.
                get({
                    let state = self.state.clone();
                    let ws_manager = self.ws_manager.take().unwrap(); // Only one instance required.
                    move |param1, param2, param3| {
                        api::common::get_connect_websocket(
                            param1, param2, param3, ws_manager, state,
                        )
                    }
                }),
            )
            .route(
                api::common::PATH_GET_VERSION,
                get({
                    let state = self.state.clone();
                    move || api::common::get_version(state)
                }),
            );

        public.merge(ConnectedApp::new(self.state.clone()).private_common_router())
    }

    pub fn create_account_server_router(&self) -> Router {
        let public = Router::new().route(
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
