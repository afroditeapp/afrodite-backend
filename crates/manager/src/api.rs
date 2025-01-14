//! HTTP API types and request handlers for all servers.

use manager_model as model;
pub use utils::SecurityApiTokenDefault;
use utoipa::OpenApi;

use crate::{
    config::Config,
    server::{client::ApiManager, update::UpdateManagerHandle},
};

// Routes
pub mod manager;

pub mod utils;

// Paths

pub const PATH_PREFIX: &str = "/api/v1/";

// API docs

#[derive(OpenApi)]
#[openapi(
    paths(
        manager::get_encryption_key,
        manager::get_software_info,
        manager::get_latest_software,
        manager::get_system_info,
        manager::get_system_info_all,
        manager::post_request_software_update,
        manager::post_request_restart_or_reset_backend,
    ),
    components(schemas(
        model::DataEncryptionKey,
        model::ServerNameText,
        model::SoftwareOptions,
        model::SoftwareOptionsQueryParam,
        model::DownloadType,
        model::DownloadTypeQueryParam,
        model::RebootQueryParam,
        model::ResetDataQueryParam,
        model::SoftwareInfo,
        model::BuildInfo,
        model::SystemInfoList,
        model::SystemInfo,
        model::CommandOutput,
    )),
    modifiers(&SecurityApiTokenDefault),
    info(
        title = "afrodite-manager",
        description = "Afrodite manager API",
        version = "0.1.0",
        license(
            name = "",
            url = "https://example.com",
        ),
    ),
)]
pub struct ManagerApiDoc;

impl ManagerApiDoc {
    pub fn api_doc_json() -> Result<String, serde_json::Error> {
        Self::openapi().to_pretty_json()
    }
}

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
