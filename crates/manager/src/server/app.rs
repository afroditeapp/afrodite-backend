use std::sync::Arc;

use axum::Router;

use self::private_routers::PrivateRoutes;
use super::{
    client::{ApiClient, ApiManager},
    update::UpdateManagerHandle,
};
use crate::{
    api::{GetApiManager, GetConfig, GetUpdateManager},
    config::Config,
};

pub mod private_routers;

#[derive(Clone)]
pub struct AppState {
    config: Arc<Config>,
    api: Arc<ApiClient>,
    update_manager: Arc<UpdateManagerHandle>,
}

impl GetConfig for AppState {
    fn config(&self) -> &Config {
        &self.config
    }
}

impl GetUpdateManager for AppState {
    fn update_manager(&self) -> &super::update::UpdateManagerHandle {
        &self.update_manager
    }
}

impl GetApiManager for AppState {
    fn api_manager(&self) -> ApiManager<'_> {
        ApiManager::new(&self.config, &self.api)
    }
}

pub struct App {
    pub state: AppState,
}

impl App {
    pub async fn new(
        config: Arc<Config>,
        api_client: Arc<ApiClient>,
        update_manager: Arc<UpdateManagerHandle>,
    ) -> Self {
        let state = AppState {
            config: config.clone(),
            api: api_client.clone(),
            update_manager,
        };

        Self { state }
    }

    pub fn state(&self) -> AppState {
        self.state.clone()
    }

    pub fn create_manager_server_router(&self) -> Router {
        let public = Router::new();
        public.merge(PrivateRoutes::new(self.state.clone()).private_manager_server_router())
    }
}
