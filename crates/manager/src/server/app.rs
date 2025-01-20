use std::sync::Arc;

use super::{client::ApiManager, reboot::RebootManagerHandle, update::UpdateManagerHandle};
use crate::{
    api::{GetApiManager, GetConfig, GetRebootManager, GetUpdateManager},
    config::Config,
};

pub type S = AppState;

#[derive(Debug, Clone)]
pub struct AppState {
    config: Arc<Config>,
    update_manager: Arc<UpdateManagerHandle>,
    reboot_manager: Arc<RebootManagerHandle>,
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

impl GetRebootManager for AppState {
    fn reboot_manager(&self) -> &RebootManagerHandle {
        &self.reboot_manager
    }
}

impl GetApiManager for AppState {
    fn api_manager(&self) -> super::client::ApiManager {
        ApiManager::new(self)
    }
}


pub struct App {
    pub state: AppState,
}

impl App {
    pub async fn new(
        config: Arc<Config>,
        update_manager: Arc<UpdateManagerHandle>,
        reboot_manager: Arc<RebootManagerHandle>,
    ) -> Self {
        let state = AppState {
            config: config.clone(),
            update_manager,
            reboot_manager,
        };

        Self { state }
    }

    pub fn state(&self) -> AppState {
        self.state.clone()
    }
}
