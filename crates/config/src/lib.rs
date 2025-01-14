#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::large_enum_variant, clippy::manual_range_contains)]

pub mod args;
pub mod bot_config_file;
pub mod file;
pub mod file_dynamic;
pub mod file_email_content;
pub mod profile_name_allowlist;

use std::{path::Path, sync::Arc};

use args::{AppMode, ArgsConfig};
use chrono::FixedOffset;
use error_stack::{Result, ResultExt};
use file::{AccountLimitsConfig, ChatLimitsConfig, DemoModeConfig, GrantAdminAccessConfig, MediaLimitsConfig};
use file_dynamic::ConfigFileDynamic;
use file_email_content::EmailContentFile;
use model::BotConfig;
use model_server_data::{AttributesFileInternal, ProfileAttributesInternal};
use profile_name_allowlist::{ProfileNameAllowlistBuilder, ProfileNameAllowlistData};
use reqwest::Url;
use sha2::{Digest, Sha256};
use simple_backend_config::SimpleBackendConfig;
use simple_backend_utils::{ContextExt, IntoReportFromString};

use self::file::{Components, ConfigFile, ExternalServices, LocationConfig};

pub const DATABASE_MESSAGE_CHANNEL_BUFFER: usize = 32;

#[derive(thiserror::Error, Debug)]
pub enum GetConfigError {
    #[error("Simple backend error")]
    SimpleBackendError,

    #[error("Get working directory error")]
    GetWorkingDir,
    #[error("File loading failed")]
    LoadFileError,
    #[error("Profile name allowlist error")]
    ProfileNameAllowlistError,

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
    components: Components,

    // Other configs
    mode: Option<AppMode>,
    profile_attributes: Option<ProfileAttributesInternal>,
    profile_attributes_sha256: Option<String>,
    email_content: Option<EmailContentFile>,

    reset_likes_utc_offset: FixedOffset,
    profile_name_allowlist: ProfileNameAllowlistData,
}

impl Config {
    pub fn minimal_config_for_api_doc_json(
        simple_backend_config: Arc<SimpleBackendConfig>,
    ) -> Self {
        Self {
            file: ConfigFile::minimal_config_for_api_doc_json(),
            file_dynamic: ConfigFileDynamic::minimal_config_for_api_doc_json(),
            simple_backend_config,
            external_services: ExternalServices::default(),
            client_api_urls: InternalApiUrls::new(None, None),
            components: Components::default(),
            mode: None,
            profile_attributes: None,
            profile_attributes_sha256: None,
            email_content: None,
            reset_likes_utc_offset: FixedOffset::east_opt(0).unwrap(),
            profile_name_allowlist: ProfileNameAllowlistData::default(),
        }
    }

    pub fn components(&self) -> Components {
        self.components
    }

    pub fn location(&self) -> LocationConfig {
        self.file.location.clone().unwrap_or_default()
    }

    /// Server should run in debug mode.
    ///
    /// Debug mode changes:
    /// * Completing initial setup will check only email when adding admin permissions.
    ///   Normally it also requires Google Account ID.
    /// * Routes for only related to benchmarking are available.
    /// * Axum JSON extractor shows errors.
    /// * Admin bot profile text moderation saves LLM response text to server
    ///   when the text is rejected.
    /// * Allow disabling some server component. This enables running the
    ///   server in microservice mode but the mode is unsupported
    ///   and currently broken.
    ///
    /// Check also [SimpleBackendConfig::debug_mode].
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

    pub fn grant_admin_access_config(&self) -> Option<&GrantAdminAccessConfig> {
        self.file.grant_admin_access.as_ref()
    }

    pub fn bot_config(&self) -> Option<&BotConfig> {
        self.file_dynamic.backend_config.bots.as_ref()
    }

    pub fn bot_config_file(&self) -> Option<&Path> {
        self.file.bot_config_file.as_deref()
    }

    pub fn limits_account(&self) -> AccountLimitsConfig {
        self.file.limits.as_ref().and_then(|v| v.account.as_ref().cloned()).unwrap_or_default()
    }

    pub fn limits_chat(&self) -> ChatLimitsConfig {
        self.file.limits.as_ref().and_then(|v| v.chat.as_ref().cloned()).unwrap_or_default()
    }

    pub fn limits_media(&self) -> MediaLimitsConfig {
        self.file.limits.as_ref().and_then(|v| v.media.as_ref().cloned()).unwrap_or_default()
    }

    pub fn profile_attributes(&self) -> Option<&ProfileAttributesInternal> {
        self.profile_attributes.as_ref()
    }

    pub fn profile_attributes_sha256(&self) -> Option<&str> {
        self.profile_attributes_sha256.as_deref()
    }

    pub fn email_content(&self) -> Option<&EmailContentFile> {
        self.email_content.as_ref()
    }

    pub fn demo_mode_config(&self) -> Option<&Vec<DemoModeConfig>> {
        self.file.demo_mode.as_ref()
    }

    pub fn simple_backend(&self) -> &SimpleBackendConfig {
        &self.simple_backend_config
    }

    pub fn simple_backend_arc(&self) -> Arc<SimpleBackendConfig> {
        self.simple_backend_config.clone()
    }

    pub fn reset_likes_utc_offset(&self) -> FixedOffset {
        self.reset_likes_utc_offset
    }

    pub fn profile_name_allowlist(&self) -> &ProfileNameAllowlistData {
        &self.profile_name_allowlist
    }

    pub fn api_obfuscation_salt(&self) -> Option<&str> {
        self.file.api_obfuscation_salt.as_deref()
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
        backend_semver_version,
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

    let components = file_config.components.unwrap_or(Components::all_enabled());

    if components != Components::all_enabled() && !simple_backend_config.debug_mode() {
        return Err(GetConfigError::InvalidConfiguration).attach_printable(
            "Disabling some server component is possible only in debug mode",
        );
    }

    let client_api_urls = create_client_api_urls(&components, &external_services)?;

    let file_dynamic =
        ConfigFileDynamic::load(current_dir).change_context(GetConfigError::LoadFileError)?;

    let (profile_attributes, profile_attributes_sha256) =
        if let Some(path) = &file_config.profile_attributes_file {
            let attributes =
                std::fs::read_to_string(path).change_context(GetConfigError::LoadFileError)?;
            let profile_attributes_sha256 = format!("{:x}", Sha256::digest(attributes.as_bytes()));
            let attributes: AttributesFileInternal =
                toml::from_str(&attributes).change_context(GetConfigError::InvalidConfiguration)?;
            let attributes = attributes
                .validate()
                .into_error_string(GetConfigError::InvalidConfiguration)?;
            (Some(attributes), Some(profile_attributes_sha256))
        } else {
            (None, None)
        };

    let email_content = if let Some(path) = &file_config.email_content_file {
        let email_content =
            EmailContentFile::load(path).change_context(GetConfigError::LoadFileError)?;
        Some(email_content)
    } else {
        None
    };

    if simple_backend_config.email_sending().is_some() && email_content.is_none() {
        return Err(GetConfigError::InvalidConfiguration).attach_printable(
            "When email sending is enabled, the email content config must exists",
        );
    }

    let limits = file_config.limits.as_ref().and_then(|v| v.chat.clone()).unwrap_or_default();
    let offset_hours = 60 * 60 * Into::<i32>::into(limits.like_limit_reset_time_utc_offset_hours);
    let Some(reset_likes_utc_offset) = FixedOffset::east_opt(offset_hours) else {
        return Err(GetConfigError::InvalidConfiguration)
            .attach_printable("like_limit_reset_time_utc_offset_hours is not valid");
    };

    let mut allowlist_builder = ProfileNameAllowlistBuilder::default();
    let csv_configs = file_config
        .profile_name_allowlist
        .as_ref()
        .map(|v| v.iter())
        .unwrap_or_default();
    for c in csv_configs {
        allowlist_builder
            .load(c)
            .change_context(GetConfigError::ProfileNameAllowlistError)?;
    }
    let profile_name_allowlist = allowlist_builder.build();

    let config = Config {
        simple_backend_config: simple_backend_config.into(),
        file: file_config,
        file_dynamic,
        external_services,
        client_api_urls,
        components,
        mode: args_config.mode.clone(),
        profile_attributes,
        profile_attributes_sha256,
        email_content,
        reset_likes_utc_offset,
        profile_name_allowlist,
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
