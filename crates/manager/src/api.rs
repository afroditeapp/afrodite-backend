//! HTTP API types and request handlers for all servers.

use crate::{
    config::Config,
    server::{client::ApiManager, reboot::RebootManagerHandle, update::UpdateManagerHandle},
};

pub mod server;
pub mod client;
pub mod utils;

// App state getters

pub trait GetConfig {
    fn config(&self) -> &Config;
}

pub trait GetApiManager {
    fn api_manager(&self) -> ApiManager;
}

pub trait GetUpdateManager {
    fn update_manager(&self) -> &UpdateManagerHandle;
}

pub trait GetRebootManager {
    fn reboot_manager(&self) -> &RebootManagerHandle;
}
