pub mod args;
pub mod file;

use std::path::{Path, PathBuf};

use error_stack::{IntoReport, Result, ResultExt};
use reqwest::Url;

use crate::{
    utils::IntoReportExt,
};

use self::{
    args::TestMode,
    file::{Components, ConfigFile, ExternalServices, SocketConfig},
};

pub const DATABASE_MESSAGE_CHANNEL_BUFFER: usize = 32;

#[derive(thiserror::Error, Debug)]
pub enum GetConfigError {
    #[error("Get working directory error")]
    GetWorkingDir,
    #[error("File loading failed")]
    LoadFileError,
    #[error("Load config file")]
    LoadConfig,

    // External service configuration errors
    #[error(
        "External service 'account internal' is required because account component is disabled."
    )]
    ExternalServiceAccountInternalMissing,
    #[error(
        "External service 'media internal' is required because media component is disabled."
    )]
    ExternalServiceMediaInternalMissing,
}

pub struct Config {
    file: ConfigFile,

    // Server related configs
    database: PathBuf,
    external_services: ExternalServices,
    client_api_urls: InternalApiUrls,

    // Other configs
    test_mode: Option<TestMode>,
}

impl Config {
    pub fn database_dir(&self) -> &Path {
        &self.database
    }

    pub fn components(&self) -> &Components {
        &self.file.components
    }

    pub fn socket(&self) -> &SocketConfig {
        &self.file.socket
    }

    /// Server should run in debug mode.
    ///
    /// Debug mode changes:
    /// * Swagger UI is enabled.
    /// * Internal API is available at same port as the public API.
    pub fn debug_mode(&self) -> bool {
        self.file.debug.unwrap_or(false)
    }

    pub fn external_services(&self) -> &ExternalServices {
        &self.external_services
    }

    pub fn external_service_urls(&self) -> &InternalApiUrls {
        &self.client_api_urls
    }

    /// Launch testing and benchmark mode instead of the server mode.
    pub fn test_mode(&self) -> Option<TestMode> {
        self.test_mode.clone()
    }
}

pub fn get_config() -> Result<Config, GetConfigError> {
    let current_dir = std::env::current_dir().into_error(GetConfigError::GetWorkingDir)?;
    let mut file_config =
        file::ConfigFile::load(current_dir).change_context(GetConfigError::LoadFileError)?;
    let args_config = args::get_config();

    let database = if let Some(database) = args_config.database_dir {
        database
    } else {
        file_config.database.dir.clone()
    };

    let external_services = file_config.external_services.take().unwrap_or_default();

    let client_api_urls = create_client_api_urls(&file_config.components, &external_services)?;

    Ok(Config {
        file: file_config,
        database,
        external_services,
        client_api_urls,
        test_mode: args_config.test_mode,
    })
}

#[derive(Debug, Clone)]
pub struct InternalApiUrls {
    pub account_base_url: Option<Url>,
    pub media_base_url: Option<Url>,
}

impl InternalApiUrls {
    pub fn new(account_base_url: Option<Url>, media_base_url: Option<Url>) -> Self { Self { account_base_url, media_base_url } }

}


pub fn create_client_api_urls(
    components: &Components,
    external_services: &ExternalServices,
) -> Result<InternalApiUrls, GetConfigError> {
    let account_internal = if !components.account {
        let url = external_services
            .account_internal
            .as_ref()
            .ok_or(GetConfigError::ExternalServiceAccountInternalMissing)
            .into_report()?;
        Some(url.clone())
    } else {
        None
    };

    let media_internal = if !components.media && components.account {
        let url = external_services
            .media_internal
            .as_ref()
            .ok_or(GetConfigError::ExternalServiceMediaInternalMissing)
            .into_report()?;
        Some(url.clone())
    } else {
        None
    };

    Ok(InternalApiUrls {
        account_base_url: account_internal,
        media_base_url: media_internal,
    })
}
