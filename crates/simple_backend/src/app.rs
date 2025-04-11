use std::sync::Arc;

use error_stack::ResultExt;
use manager_api::{ClientError, ManagerClientWithRequestReceiver};
use manager_model::ManagerInstanceNameValue;
use simple_backend_config::SimpleBackendConfig;

use super::manager_client::ManagerApiClient;
use crate::{
    file_package::FilePackageManager, map::TileMapManager, maxmind_db::MaxMindDbManagerData, perf::PerfMetricsManagerData, sign_in_with::SignInWithManager
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
    pub maxmind_db: Arc<MaxMindDbManagerData>,
}

impl SimpleBackendAppState {
    pub async fn new(
        config: Arc<SimpleBackendConfig>,
        perf_data: Arc<PerfMetricsManagerData>,
        manager_api: Arc<ManagerApiClient>,
        maxmind_db: Arc<MaxMindDbManagerData>,
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
            maxmind_db,
        })
    }
}

pub trait SignInWith {
    fn sign_in_with_manager(&self) -> &SignInWithManager;
}

pub trait GetManagerApi {
    fn manager_api_client(&self) -> &ManagerApiClient;

    async fn manager_request(
        &self
    ) -> error_stack::Result<ManagerClientWithRequestReceiver, ClientError> {
        self.manager_api_client().new_request().await
    }

    async fn manager_request_to(
        &self,
        name: ManagerInstanceNameValue,
    ) -> error_stack::Result<ManagerClientWithRequestReceiver, ClientError> {
        self.manager_api_client().new_request_to_instance(name.manager_name).await
    }
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

pub trait MaxMindDbDataProvider {
    fn maxmind_db(&self) -> &MaxMindDbManagerData;
}
