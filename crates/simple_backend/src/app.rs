use std::sync::Arc;

use simple_backend_config::SimpleBackendConfig;

use super::manager_client::{ManagerApiClient, ManagerApiManager};
use crate::{manager_client::ManagerClientError, map::TileMapManager, perf::PerfCounterManagerData, sign_in_with::SignInWithManager};

#[derive(Clone)]
pub struct SimpleBackendAppState {
    pub manager_api: Arc<ManagerApiClient>,
    pub config: Arc<SimpleBackendConfig>,
    pub sign_in_with: Arc<SignInWithManager>,
    pub tile_map: Arc<TileMapManager>,
    pub perf_data: Arc<PerfCounterManagerData>,
}

impl SimpleBackendAppState {
    pub fn new(
        config: Arc<SimpleBackendConfig>,
        perf_data: Arc<PerfCounterManagerData>,
    ) -> error_stack::Result<Self, ManagerClientError> {
        let manager_api = ManagerApiClient::new(&config)?.into();
        Ok(SimpleBackendAppState {
            tile_map: TileMapManager::new(&config).into(),
            sign_in_with: SignInWithManager::new(config.clone()).into(),
            config,
            manager_api,
            perf_data,
        })
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
