use std::sync::Arc;

use error_stack::ResultExt;
use manager_api::{ClientError, ManagerClientWithRequestReceiver};
use simple_backend_config::SimpleBackendConfig;

use super::manager_client::ManagerApiClient;
use crate::{
    file_package::FilePackageManager, map::TileMapManager, perf::PerfMetricsManagerData,
    sign_in_with::SignInWithManager,
};

#[derive(thiserror::Error, Debug)]
pub enum AppStateCreationError {
    #[error("File package manager error")]
    FilePackageManagerError,
}

#[derive(Clone)]
pub struct SimpleBackendAppState {
    pub manager_api: Arc<ManagerApiClient>,
    pub config: Arc<SimpleBackendConfig>,
    pub sign_in_with: Arc<SignInWithManager>,
    pub tile_map: Arc<TileMapManager>,
    pub perf_data: Arc<PerfMetricsManagerData>,
    pub file_packages: Arc<FilePackageManager>,
}

impl SimpleBackendAppState {
    pub async fn new(
        config: Arc<SimpleBackendConfig>,
        perf_data: Arc<PerfMetricsManagerData>,
        manager_api: Arc<ManagerApiClient>,
    ) -> error_stack::Result<Self, AppStateCreationError> {
        Ok(SimpleBackendAppState {
            tile_map: TileMapManager::new(&config).into(),
            sign_in_with: SignInWithManager::new(config.clone()).into(),
            file_packages: FilePackageManager::new(&config)
                .await
                .change_context(AppStateCreationError::FilePackageManagerError)?
                .into(),
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
    async fn manager_api(&self) -> error_stack::Result<ManagerClientWithRequestReceiver, ClientError>;
}

pub trait GetSimpleBackendConfig {
    fn simple_backend_config(&self) -> &SimpleBackendConfig;
}

pub trait GetTileMap {
    fn tile_map(&self) -> &TileMapManager;
}

pub trait PerfCounterDataProvider {
    fn perf_counter_data(&self) -> &PerfMetricsManagerData;
}

pub trait FilePackageProvider {
    fn file_package(&self) -> &FilePackageManager;
}
