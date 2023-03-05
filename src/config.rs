pub mod args;
pub mod file;

use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
    path::{Path, PathBuf}, sync::Arc,
};

use clap::{arg, command, value_parser};

use error_stack::{Report, Result, ResultExt, IntoReport};
use serde::Deserialize;

use crate::{utils::IntoReportExt, client::{account::AccountInternalApiUrls, media::MediaInternalApiUrls}};

use self::{
    args::{ArgsConfig, ServerComponent},
    file::{Components, ConfigFile, ConfigFileError, SocketConfig, ExternalServices},
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

    #[error("External service 'account internal' is required because account component is disabled.")]
    ExternalServiceAccountInternalMissing,
}

#[derive(Debug)]
pub struct ClientApiUrls {
    pub account_internal: AccountInternalApiUrls,
    pub media_internal: MediaInternalApiUrls,
}

pub struct Config {
    file: ConfigFile,
    database: PathBuf,
    external_services: ExternalServices,
    client_api_urls: Arc<ClientApiUrls>,
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

    pub fn external_service_urls(&self) -> Arc<ClientApiUrls> {
        self.client_api_urls.clone()
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

    let client_api_urls =
        create_client_api_urls(
            &file_config.components,
            &external_services,
        )?;

    Ok(Config {
        file: file_config,
        database,
        external_services,
        client_api_urls: Arc::new(client_api_urls),
    })
}

pub fn create_client_api_urls(
    components: &Components,
    external_services: &ExternalServices,
) -> Result<ClientApiUrls, GetConfigError> {
    let account_internal = if !components.account {
        let url = external_services
            .account_internal
            .as_ref()
            .ok_or(GetConfigError::ExternalServiceAccountInternalMissing)
            .into_report()?;
        AccountInternalApiUrls::new(url.clone())
            .change_context(GetConfigError::ExternalServiceAccountInternalMissing)?
    } else {
        AccountInternalApiUrls::default()
    };


    Ok(ClientApiUrls {
        account_internal,
        media_internal: MediaInternalApiUrls::default(),
    })
}
