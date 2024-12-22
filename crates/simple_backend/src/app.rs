use std::sync::Arc;

use error_stack::ResultExt;
use simple_backend_config::SimpleBackendConfig;
use simple_backend_utils::IntoReportFromString;

use super::manager_client::{ManagerApiClient, ManagerApiManager};
use crate::{
    file_package::FilePackageManager, map::TileMapManager, perf::PerfMetricsManagerData,
    sign_in_with::SignInWithManager,
};

#[derive(thiserror::Error, Debug)]
pub enum AppStateCreationError {
    #[error("Manager client creation error")]
    ManagerClientError,

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
    ) -> error_stack::Result<Self, AppStateCreationError> {
        let manager_api = ManagerApiClient::new(&config)
            .into_error_string(AppStateCreationError::ManagerClientError)?
            .into();
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
    fn manager_api(&self) -> ManagerApiManager;
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
