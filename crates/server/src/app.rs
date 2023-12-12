use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use config::{file::ConfigFileError, file_dynamic::ConfigFileDynamic, Config};
use error_stack::{Result, ResultExt};
use futures::Future;
use model::{AccountId, BackendConfig, BackendVersion};
use simple_backend::{
    app::{GetSimpleBackendConfig, SimpleBackendAppState},
    web_socket::WebSocketManager,
};

use self::routes_connected::ConnectedApp;
use super::{
    data::{
        read::ReadCommands,
        utils::{AccessTokenManager, AccountIdManager},
        write_commands::{WriteCmds, WriteCommandRunnerHandle},
        DataError, RouterDatabaseReadHandle, RouterDatabaseWriteHandle,
    },
    internal::{InternalApiClient, InternalApiManager},
};
use crate::{
    api,
    data::write_concurrent::{ConcurrentWriteAction, ConcurrentWriteSelectorHandle},
    event::EventManager,
};

pub mod routes_connected;
pub mod routes_internal;

#[derive(Clone)]
pub struct AppState {
    database: Arc<RouterDatabaseReadHandle>,
    write_queue: Arc<WriteCommandRunnerHandle>,
    internal_api: Arc<InternalApiClient>,
    config: Arc<Config>,
    events: Arc<EventManager>,
}

pub trait GetAccessTokens {
    /// Users which are logged in.
    fn access_tokens(&self) -> AccessTokenManager<'_>;
}

impl GetAccessTokens for SimpleBackendAppState<AppState> {
    fn access_tokens(&self) -> AccessTokenManager<'_> {
        self.business_logic_state().database.access_token_manager()
    }
}

pub trait GetAccounts {
    /// All accounts registered in the service.
    fn accounts(&self) -> AccountIdManager<'_>;
}

impl GetAccounts for SimpleBackendAppState<AppState> {
    fn accounts(&self) -> AccountIdManager<'_> {
        self.business_logic_state().database.account_id_manager()
    }
}

#[async_trait::async_trait]
pub trait WriteData {
    async fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = error_stack::Result<CmdResult, DataError>> + Send + 'static,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        cmd: GetCmd,
    ) -> error_stack::Result<CmdResult, DataError>;

    async fn write_concurrent<
        CmdResult: Send + 'static,
        Cmd: Future<Output = ConcurrentWriteAction<CmdResult>> + Send + 'static,
        GetCmd: FnOnce(ConcurrentWriteSelectorHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountId,
        cmd: GetCmd,
    ) -> error_stack::Result<CmdResult, DataError>;
}

#[async_trait::async_trait]
impl WriteData for SimpleBackendAppState<AppState> {
    async fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = Result<CmdResult, DataError>> + Send + 'static,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        cmd: GetCmd,
    ) -> Result<CmdResult, DataError> {
        self.business_logic_state().write_queue.write(cmd).await
    }

    async fn write_concurrent<
        CmdResult: Send + 'static,
        Cmd: Future<Output = ConcurrentWriteAction<CmdResult>> + Send + 'static,
        GetCmd: FnOnce(ConcurrentWriteSelectorHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountId,
        cmd: GetCmd,
    ) -> Result<CmdResult, DataError> {
        self.business_logic_state()
            .write_queue
            .concurrent_write(account, cmd)
            .await
    }
}

pub trait ReadData {
    fn read(&self) -> ReadCommands<'_>;
}

impl ReadData for SimpleBackendAppState<AppState> {
    fn read(&self) -> ReadCommands<'_> {
        self.business_logic_state().database.read()
    }
}

pub trait GetInternalApi {
    fn internal_api(&self) -> InternalApiManager<Self>
    where
        Self: Sized;
}

impl GetInternalApi for SimpleBackendAppState<AppState> {
    fn internal_api(&self) -> InternalApiManager<Self> {
        InternalApiManager::new(self, &self.business_logic_state().internal_api)
    }
}

pub trait GetConfig {
    fn config(&self) -> &Config;
}

impl GetConfig for SimpleBackendAppState<AppState> {
    fn config(&self) -> &Config {
        &self.business_logic_state().config
    }
}

#[async_trait::async_trait]
pub trait WriteDynamicConfig {
    async fn write_config(&self, config: BackendConfig)
        -> error_stack::Result<(), ConfigFileError>;
}

#[async_trait::async_trait]
impl WriteDynamicConfig for SimpleBackendAppState<AppState> {
    async fn write_config(
        &self,
        config: BackendConfig,
    ) -> error_stack::Result<(), ConfigFileError> {
        tokio::task::spawn_blocking(move || {
            if let Some(bots) = config.bots {
                ConfigFileDynamic::edit_bot_config_from_current_dir(bots)?
            }

            Result::<(), ConfigFileError>::Ok(())
        })
        .await
        .change_context(ConfigFileError::LoadConfig)??;

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait ReadDynamicConfig {
    async fn read_config(&self) -> error_stack::Result<BackendConfig, ConfigFileError>;
}

#[async_trait::async_trait]
impl ReadDynamicConfig for SimpleBackendAppState<AppState> {
    async fn read_config(&self) -> error_stack::Result<BackendConfig, ConfigFileError> {
        let config =
            tokio::task::spawn_blocking(move || ConfigFileDynamic::load_from_current_dir())
                .await
                .change_context(ConfigFileError::LoadConfig)??;

        Ok(config.backend_config)
    }
}

pub trait BackendVersionProvider {
    fn backend_version(&self) -> BackendVersion;
}

impl BackendVersionProvider for SimpleBackendAppState<AppState> {
    fn backend_version(&self) -> BackendVersion {
        BackendVersion {
            backend_code_version: self
                .simple_backend_config()
                .backend_code_version()
                .to_string(),
            backend_version: self
                .simple_backend_config()
                .backend_semver_version()
                .to_string(),
            protocol_version: "1.0.0".to_string(),
        }
    }
}

pub trait EventManagerProvider {
    fn event_manager(&self) -> &EventManager;
}

impl EventManagerProvider for SimpleBackendAppState<AppState> {
    fn event_manager(&self) -> &EventManager {
        &self.business_logic_state().events
    }
}

pub struct App {
    state: SimpleBackendAppState<AppState>,
    web_socket_manager: Option<WebSocketManager>,
}

impl App {
    pub async fn create_app_state(
        database_handle: RouterDatabaseReadHandle,
        _database_write_handle: RouterDatabaseWriteHandle,
        write_queue: WriteCommandRunnerHandle,
        config: Arc<Config>,
    ) -> AppState {
        let database = Arc::new(database_handle);
        let state = AppState {
            config: config.clone(),
            database: database.clone(),
            write_queue: Arc::new(write_queue),
            internal_api: InternalApiClient::new(config.external_service_urls().clone()).into(),
            events: EventManager::new(database).into(),
        };

        state
    }

    pub fn new(
        state: SimpleBackendAppState<AppState>,
        web_socket_manager: WebSocketManager,
    ) -> Self {
        Self {
            state,
            web_socket_manager: web_socket_manager.into(),
        }
    }

    pub fn state(&self) -> SimpleBackendAppState<AppState> {
        self.state.clone()
    }

    pub fn create_common_server_router(&mut self) -> Router {
        let public = Router::new()
            .route(
                api::common::PATH_CONNECT, // This route checks the access token by itself.
                get({
                    let state = self.state.clone();
                    let ws_manager = self
                        .web_socket_manager
                        .take()
                        .expect("This should be called only once");
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
