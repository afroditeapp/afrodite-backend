use std::sync::Arc;

use super::update::UpdateManagerHandle;
use crate::{
    api::{GetConfig, GetUpdateManager},
    config::Config,
};

pub type S = AppState;

#[derive(Clone)]
pub struct AppState {
    config: Arc<Config>,
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


pub struct App {
    pub state: AppState,
}

impl App {
    pub async fn new(
        config: Arc<Config>,
        update_manager: Arc<UpdateManagerHandle>,
    ) -> Self {
        let state = AppState {
            config: config.clone(),
            update_manager,
        };

        Self { state }
    }

    pub fn state(&self) -> AppState {
        self.state.clone()
    }
}
