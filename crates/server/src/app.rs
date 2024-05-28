use std::{net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use config::{file::ConfigFileError, file_dynamic::ConfigFileDynamic, Config};
use database::current::write::chat::PushNotificationStateInfo;
use error_stack::{Result, ResultExt};

use futures::Future;
use model::{AccessToken, AccountId, AccountIdInternal, AccountState, BackendConfig, BackendVersion, Capabilities, PendingNotificationFlags};
use server_api::{account::demo_mode_router, internal_api::InternalApiClient};
use server_common::push_notifications::{PushNotificationError, PushNotificationSender, PushNotificationStateProvider};
use simple_backend::{
    app::{GetManagerApi, GetSimpleBackendConfig, GetTileMap, PerfCounterDataProvider, SignInWith, SimpleBackendAppState}, manager_client::ManagerApiManager, map::TileMapManager, perf::PerfCounterManagerData, sign_in_with::SignInWithManager, web_socket::WebSocketManager
};
use simple_backend_config::SimpleBackendConfig;

use self::routes_connected::ConnectedApp;
use server_data::{
    content_processing::ContentProcessingManagerData, demo::DemoModeManager, read::ReadCommands, utils::{AccessTokenManager, AccountIdManager}, write_commands::{WriteCmds, WriteCommandRunnerHandle}, write_concurrent::{ConcurrentWriteAction, ConcurrentWriteSelectorHandle}, DataError, RouterDatabaseReadHandle,
    event::{EventManagerWithCacheReference},
};

pub use server_api::app::*;


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
// Server common

impl EventManagerProvider for S {
    fn event_manager(&self) -> EventManagerWithCacheReference<'_> {
        EventManagerWithCacheReference::new(
            self.database.cache(),
            &self.push_notification_sender,
        )
    }
}

impl GetAccounts for S {
    async fn get_internal_id(
        &self,
        id: AccountId
    ) -> Result<AccountIdInternal, DataError> {
        self.database
            .account_id_manager()
            .get_internal_id(id)
            .await
            .map_err(|e| e.into_report())
    }
}

#[async_trait::async_trait]
impl ReadDynamicConfig for S {
    async fn read_config(&self) -> error_stack::Result<BackendConfig, ConfigFileError> {
        let config =
            tokio::task::spawn_blocking(ConfigFileDynamic::load_from_current_dir)
                .await
                .change_context(ConfigFileError::LoadConfig)??;

        Ok(config.backend_config)
    }
}

impl BackendVersionProvider for S {
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

impl GetConfig for S {
    fn config(&self) -> &Config {
        &self.config
    }
}

#[async_trait::async_trait]
impl WriteDynamicConfig for S {
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

impl PushNotificationStateProvider for S {
    async fn get_push_notification_state_info_and_add_notification_value(
        &self,
        account_id: AccountIdInternal,
        flags: PendingNotificationFlags,
    ) -> Result<PushNotificationStateInfo, PushNotificationError> {
        self
            .write(move |cmds| async move {
                cmds.chat()
                    .push_notifications()
                    .get_push_notification_state_info_and_add_notification_value(
                        account_id,
                        flags.into(),
                    )
                    .await
            })
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::SettingPushNotificationSentFlagFailed)
    }

    async fn enable_push_notification_sent_flag(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<(), PushNotificationError> {
        self
            .write(move |cmds| async move {
                cmds.chat()
                    .push_notifications()
                    .enable_push_notification_sent_flag(account_id)
                    .await
            })
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::SettingPushNotificationSentFlagFailed)
    }

    async fn remove_device_token(
        &self,
        account_id: AccountIdInternal
    ) -> Result<(), PushNotificationError> {
        self
            .write(move |cmds| async move {
                cmds.chat()
                    .push_notifications()
                    .remove_device_token(account_id)
                    .await
            })
            .await
            .map_err(|e| e.into_report())
            .change_context(PushNotificationError::RemoveDeviceTokenFailed)
    }
}

// Server data

#[async_trait::async_trait]
impl WriteData for S {
    async fn write<
        CmdResult: Send + 'static,
        Cmd: Future<Output = server_common::result::Result<CmdResult, DataError>> + Send + 'static,
        GetCmd: FnOnce(WriteCmds) -> Cmd + Send + 'static,
    >(
        &self,
        cmd: GetCmd,
    ) -> server_common::result::Result<CmdResult, DataError> {
        self.write_queue.write(cmd).await
    }

    async fn write_concurrent<
        CmdResult: Send + 'static,
        Cmd: Future<Output = ConcurrentWriteAction<CmdResult>> + Send + 'static,
        GetCmd: FnOnce(ConcurrentWriteSelectorHandle) -> Cmd + Send + 'static,
    >(
        &self,
        account: AccountId,
        cmd: GetCmd,
    ) -> server_common::result::Result<CmdResult, DataError> {
        self
            .write_queue
            .concurrent_write(account, cmd)
            .await
    }
}

impl ReadData for S {
    fn read(&self) -> ReadCommands<'_> {
        self.database.read()
    }
}

// Server API

impl StateBase for AppState {}

impl GetInternalApi for S {
    fn internal_api_client(&self) -> &InternalApiClient {
        &self.internal_api
    }
}

impl GetAccessTokens for S {
    async fn access_token_exists(&self, token: &AccessToken) -> Option<AccountIdInternal> {
        self.database.access_token_manager().access_token_exists(token).await
    }

    async fn access_token_and_connection_exists(
        &self,
        token: &AccessToken,
        connection: SocketAddr,
    ) -> Option<(AccountIdInternal, Capabilities, AccountState)> {
        self.database.access_token_manager().access_token_and_connection_exists(token, connection).await
    }
}

impl ContentProcessingProvider for S {
    fn content_processing(&self) -> &ContentProcessingManagerData {
        &self.content_processing
    }
}

impl DemoModeManagerProvider for S {
    fn demo_mode(&self) -> &DemoModeManager {
        &self.demo_mode
    }
}

// Simple backend

impl SignInWith for S {
    fn sign_in_with_manager(&self) -> &SignInWithManager {
        &self.simple_backend_state.sign_in_with
    }
}

impl GetManagerApi for S {
    fn manager_api(&self) -> ManagerApiManager {
        ManagerApiManager::new(&self.simple_backend_state.manager_api)
    }
}

impl GetSimpleBackendConfig for S {
    fn simple_backend_config(&self) -> &SimpleBackendConfig {
        &self.simple_backend_state.config
    }
}

impl GetTileMap for S {
    fn tile_map(&self) -> &TileMapManager {
        &self.simple_backend_state.tile_map
    }
}

impl PerfCounterDataProvider for S {
    fn perf_counter_data(&self) -> &PerfCounterManagerData {
        &self.simple_backend_state.perf_data
    }
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
            simple_backend_state
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
                server_api::common::PATH_CONNECT, // This route checks the access token by itself.
                get({
                    let ws_manager = self
                        .web_socket_manager
                        .take()
                        .expect("This should be called only once");
                    move |state, param1, param2, param3| {
                        server_api::common::get_connect_websocket::<S>(
                            state, param1, param2, param3, ws_manager,
                        )
                    }
                }),
            )
            .route(
                server_api::common::PATH_GET_VERSION,
                get(server_api::common::get_version::<S>),
            )
            .with_state(self.state());

        public.merge(ConnectedApp::new(self.state.clone()).private_common_router())
    }

    pub fn create_account_server_router(&self) -> Router {
        let public = Router::new()
            .route(
                server_api::account::PATH_SIGN_IN_WITH_LOGIN,
                post(server_api::account::post_sign_in_with_login::<S>),
            )
            .with_state(self.state());

        let public = if self.state.config().demo_mode_config().is_some() {
            public.merge(demo_mode_router(self.state.clone()))
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
            server_api::chat::push_notifications::push_notification_router_public(self.state.clone()),
        );

        public.merge(ConnectedApp::new(self.state.clone()).private_chat_server_router())
    }
}
