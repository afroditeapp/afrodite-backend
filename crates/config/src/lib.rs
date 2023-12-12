#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod args;
pub mod file_dynamic;
pub mod file;

use std::{
    sync::{Arc},
};

use args::{AppMode, ArgsConfig};
use error_stack::{Result, ResultExt};
use file::{QueueLimitsConfig, StaticBotConfig};
use file_dynamic::{ConfigFileDynamic};
use model::BotConfig;
use reqwest::Url;

use simple_backend_config::SimpleBackendConfig;

use simple_backend_utils::ContextExt;

use self::{
    file::{
        Components, ConfigFile, ExternalServices, InternalApiConfig,
        LocationConfig,
    },
};

pub const DATABASE_MESSAGE_CHANNEL_BUFFER: usize = 32;

#[derive(thiserror::Error, Debug)]
pub enum GetConfigError {
    #[error("Simple backend error")]
    SimpleBackendError,

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
    #[error("External service 'media internal' is required because media component is disabled.")]
    ExternalServiceMediaInternalMissing,

    #[error("Invalid configuration")]
    InvalidConfiguration,
}

#[derive(Debug)]
pub struct Config {
    file: ConfigFile,
    file_dynamic: ConfigFileDynamic,
    simple_backend_config: Arc<SimpleBackendConfig>,

    // Server related configs
    external_services: ExternalServices,
    client_api_urls: InternalApiUrls,

    // Other configs
    mode: Option<AppMode>,
}

impl Config {
    pub fn components(&self) -> &Components {
        &self.file.components
    }

    pub fn location(&self) -> LocationConfig {
        self.file.location.clone().unwrap_or_default()
    }

    /// Server should run in debug mode.
    ///
    /// Debug mode changes:
    /// * Completing initial setup will check only email when adding admin capabilities.
    ///   Normally it also requires Google Account ID.
    /// * Routes for only related to benchmarking are available.
    /// * Axum JSON extractor shows errors.
    ///
    /// Check also `SimpleBackendConfig::debug_mode`.
    pub fn debug_mode(&self) -> bool {
        self.simple_backend_config.debug_mode()
    }

    pub fn external_services(&self) -> &ExternalServices {
        &self.external_services
    }

    pub fn external_service_urls(&self) -> &InternalApiUrls {
        &self.client_api_urls
    }

    /// Server binary was launched in a special mode instead of the server mode.
    ///
    /// If None then the mode is the server mode.
    pub fn current_mode(&self) -> Option<AppMode> {
        self.mode.clone()
    }

    pub fn admin_email(&self) -> &str {
        &self.file.admin_email
    }

    pub fn internal_api_config(&self) -> InternalApiConfig {
        self.file.internal_api.clone().unwrap_or_default()
    }

    pub fn bot_config(&self) -> Option<&BotConfig> {
        self.file_dynamic.backend_config.bots.as_ref()
    }

    pub fn static_bot_config(&self) -> Option<&StaticBotConfig> {
        self.file.bots.as_ref()
    }

    pub fn queue_limits(&self) -> QueueLimitsConfig {
        self.file.queue_limits.clone().unwrap_or_default()
    }

    pub fn simple_backend(&self) -> &SimpleBackendConfig {
        &self.simple_backend_config
    }

    pub fn simple_backend_arc(&self) -> Arc<SimpleBackendConfig> {
        self.simple_backend_config.clone()
    }
}

pub fn get_config(
    args_config: ArgsConfig,
    backend_code_version: String,
    backend_semver_version: String,
) -> Result<Config, GetConfigError> {
    let simple_backend_config = simple_backend_config::get_config(
        args_config.server,
        backend_code_version,
        backend_semver_version
    )
        .change_context(GetConfigError::SimpleBackendError)?;

    let current_dir = std::env::current_dir().change_context(GetConfigError::GetWorkingDir)?;
    let file_config =
        file::ConfigFile::load(&current_dir).change_context(GetConfigError::LoadFileError)?;

    let external_services = file_config
        .external_services
        .clone()
        .take()
        .unwrap_or_default();

    let client_api_urls = create_client_api_urls(&file_config.components, &external_services)?;

    let file_dynamic =
        ConfigFileDynamic::load(current_dir).change_context(GetConfigError::LoadFileError)?;

    if file_dynamic.backend_config.bots.is_some() && !file_config.internal_api.as_ref().map(|c| c.bot_login).unwrap_or_default() {
        return Err(GetConfigError::InvalidConfiguration)
            .attach_printable("When bots are enabled, internal API bot login must be also enabled");
    }

    let config = Config {
        simple_backend_config: simple_backend_config.into(),
        file: file_config,
        file_dynamic,
        external_services,
        client_api_urls,
        mode: args_config.mode.clone(),
    };

    Ok(config)
}

#[derive(Debug, Clone)]
pub struct InternalApiUrls {
    pub account_base_url: Option<Url>,
    pub media_base_url: Option<Url>,
}

impl InternalApiUrls {
    pub fn new(account_base_url: Option<Url>, media_base_url: Option<Url>) -> Self {
        Self {
            account_base_url,
            media_base_url,
        }
    }
}

pub fn create_client_api_urls(
    components: &Components,
    external_services: &ExternalServices,
) -> Result<InternalApiUrls, GetConfigError> {
    let account_internal = if !components.account {
        let url = external_services
            .account_internal
            .as_ref()
            .ok_or(GetConfigError::ExternalServiceAccountInternalMissing.report())?;
        Some(url.clone())
    } else {
        None
    };

    let media_internal = if !components.media && components.account {
        let url = external_services
            .media_internal
            .as_ref()
            .ok_or(GetConfigError::ExternalServiceMediaInternalMissing.report())?;
        Some(url.clone())
    } else {
        None
    };

    Ok(InternalApiUrls {
        account_base_url: account_internal,
        media_base_url: media_internal,
    })
}
