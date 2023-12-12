use std::sync::Arc;


// use config::{Config, file::ConfigFileError, file_dynamic::ConfigFileDynamic, GetConfigError};
use error_stack::{Result};

// use model::{AccountId, BackendVersion, BackendConfig};
use simple_backend_config::SimpleBackendConfig;


use crate::{
    sign_in_with::SignInWithManager,
};
use super::{
    // data::{
    //     read::ReadCommands,
    //     utils::{AccessTokenManager, AccountIdManager},
    //     write_commands::{WriteCmds, WriteCommandRunnerHandle},
    //     write_concurrent::ConcurrentWriteImageHandle,
    //     DataError, RouterDatabaseReadHandle, RouterDatabaseWriteHandle,
    // },
    // internal::{InternalApiClient, InternalApiManager},
    manager_client::{ManagerApiClient, ManagerApiManager, ManagerClientError},
};


use crate::{map::TileMapManager, perf::PerfCounterManagerData};


#[derive(Clone)]
pub struct SimpleBackendAppState<
    T: Clone,
> {
    manager_api: Arc<ManagerApiClient>,
    config: Arc<SimpleBackendConfig>,
    sign_in_with: Arc<SignInWithManager>,
    tile_map: Arc<TileMapManager>,
    perf_data: Arc<PerfCounterManagerData>,
    business_logic_data: Arc<T>,
}

impl <T: Clone> SimpleBackendAppState<T> {
    pub fn business_logic_state(&self) -> &T {
        &self.business_logic_data
    }
}

pub trait SignInWith {
    fn sign_in_with_manager(&self) -> &SignInWithManager;
}

pub trait GetManagerApi {
    fn manager_api(&self) -> ManagerApiManager;
}

pub trait GetSimpleBackendConfig {
    fn simple_backend_config(&self) -> &SimpleBackendConfig;
}

pub trait GetTileMap {
    fn tile_map(&self) -> &TileMapManager;
}

pub trait PerfCounterDataProvider {
    fn perf_counter_data(&self) -> &PerfCounterManagerData;
}

impl <T: Clone> SignInWith for SimpleBackendAppState<T> {
    fn sign_in_with_manager(&self) -> &SignInWithManager {
        &self.sign_in_with
    }
}

impl <T: Clone> GetManagerApi for SimpleBackendAppState<T> {
    fn manager_api(&self) -> ManagerApiManager {
        ManagerApiManager::new(&self.manager_api)
    }
}

impl <T: Clone> GetSimpleBackendConfig for SimpleBackendAppState<T> {
    fn simple_backend_config(&self) -> &SimpleBackendConfig {
        &self.config
    }
}

impl <T: Clone> GetTileMap for SimpleBackendAppState<T> {
    fn tile_map(&self) -> &TileMapManager {
        &self.tile_map
    }
}

impl <T: Clone> PerfCounterDataProvider for SimpleBackendAppState<T> {
    fn perf_counter_data(&self) -> &PerfCounterManagerData {
        &self.perf_data
    }
}

pub struct App<T: Clone> {
    state: SimpleBackendAppState<T>,
}

impl <T: Clone> App<T> {
    pub async fn new(
        config: Arc<SimpleBackendConfig>,
        perf_data: Arc<PerfCounterManagerData>,
        business_logic_state: T,
    ) -> Result<Self, ManagerClientError> {
        let state = SimpleBackendAppState {
            config: config.clone(),
            manager_api: ManagerApiClient::new(&config)?.into(),
            tile_map: TileMapManager::new(&config).into(),
            sign_in_with: SignInWithManager::new(config).into(),
            perf_data,
            business_logic_data: Arc::new(business_logic_state),
        };

        Ok(Self {
            state,
        })
    }

    pub fn state(&self) -> SimpleBackendAppState<T> {
        self.state.clone()
    }

    pub fn into_state(self) -> SimpleBackendAppState<T> {
        self.state
    }

    // pub fn create_common_server_router(&mut self) -> Router {
    //     let public = Router::new()
    //         .route(
    //             api::common::PATH_CONNECT, // This route checks the access token by itself.
    //             get({
    //                 let state = self.state.clone();
    //                 let ws_manager = self.ws_manager.take().unwrap(); // Only one instance required.
    //                 move |param1, param2, param3| {
    //                     api::common::get_connect_websocket(
    //                         param1, param2, param3, ws_manager, state,
    //                     )
    //                 }
    //             }),
    //         )
    //         .route(
    //             api::common::PATH_GET_VERSION,
    //             get({
    //                 let state = self.state.clone();
    //                 move || api::common::get_version(state)
    //             }),
    //         );

    //     public.merge(ConnectedApp::new(self.state.clone()).private_common_router())
    // }

    // pub fn create_account_server_router(&self) -> Router {
    //     let public = Router::new().route(
    //         api::account::PATH_SIGN_IN_WITH_LOGIN,
    //         post({
    //             let state = self.state.clone();
    //             move |body| api::account::post_sign_in_with_login(body, state)
    //         }),
    //     );

    //     public.merge(ConnectedApp::new(self.state.clone()).private_account_server_router())
    // }
}
