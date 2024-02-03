use std::sync::Arc;

// use config::{Config, file::ConfigFileError, file_dynamic::ConfigFileDynamic, GetConfigError};
use error_stack::Result;
// use model::{AccountId, BackendVersion, BackendConfig};
use simple_backend_config::SimpleBackendConfig;

use super::manager_client::{ManagerApiClient, ManagerApiManager, ManagerClientError};
use crate::{map::TileMapManager, perf::PerfCounterManagerData, sign_in_with::SignInWithManager};

#[derive(Clone)]
pub struct SimpleBackendAppState<T: Clone> {
    manager_api: Arc<ManagerApiClient>,
    config: Arc<SimpleBackendConfig>,
    sign_in_with: Arc<SignInWithManager>,
    tile_map: Arc<TileMapManager>,
    perf_data: Arc<PerfCounterManagerData>,
    business_logic_data: Arc<T>,
}

impl<T: Clone> SimpleBackendAppState<T> {
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

impl<T: Clone> SignInWith for SimpleBackendAppState<T> {
    fn sign_in_with_manager(&self) -> &SignInWithManager {
        &self.sign_in_with
    }
}

impl<T: Clone> GetManagerApi for SimpleBackendAppState<T> {
    fn manager_api(&self) -> ManagerApiManager {
        ManagerApiManager::new(&self.manager_api)
    }
}

impl<T: Clone> GetSimpleBackendConfig for SimpleBackendAppState<T> {
    fn simple_backend_config(&self) -> &SimpleBackendConfig {
        &self.config
    }
}

impl<T: Clone> GetTileMap for SimpleBackendAppState<T> {
    fn tile_map(&self) -> &TileMapManager {
        &self.tile_map
    }
}

impl<T: Clone> PerfCounterDataProvider for SimpleBackendAppState<T> {
    fn perf_counter_data(&self) -> &PerfCounterManagerData {
        &self.perf_data
    }
}

pub struct StateBuilder {
    config: Arc<SimpleBackendConfig>,
    perf_data: Arc<PerfCounterManagerData>,
    manager_api: Arc<ManagerApiClient>,
}

impl StateBuilder {
    pub fn new(
        config: Arc<SimpleBackendConfig>,
        perf_data: Arc<PerfCounterManagerData>,
    ) -> Result<Self, ManagerClientError> {
        let manager_api = ManagerApiClient::new(&config)?.into();
        Ok(Self {
            config,
            perf_data,
            manager_api,
        })
    }

    pub fn build<T: Clone>(self, business_logic_state: T) -> SimpleBackendAppState<T> {
        let state = SimpleBackendAppState {
            config: self.config.clone(),
            manager_api: self.manager_api,
            tile_map: TileMapManager::new(&self.config).into(),
            sign_in_with: SignInWithManager::new(self.config).into(),
            perf_data: self.perf_data,
            business_logic_data: Arc::new(business_logic_state),
        };

        state
    }
}
