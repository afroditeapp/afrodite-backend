#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]
#![allow(clippy::large_enum_variant, clippy::manual_range_contains)]

use regex::Regex;
// Ignore unused depenency warning
use tls_client as _;

pub mod args;
pub mod bot_config_file;
pub mod csv;
pub mod file;
pub mod file_dynamic;
pub mod file_email_content;
pub mod file_notification_content;

use std::{path::Path, sync::Arc};

use args::{AppMode, ArgsConfig};
use bot_config_file::BotConfigFile;
use csv::{
    attribute_values::AttributeValuesCsvLoader,
    profile_name_allowlist::{ProfileNameAllowlistBuilder, ProfileNameAllowlistData},
};
use error_stack::{Result, ResultExt};
use file::{
    AccountLimitsConfig, AutomaticProfileSearchConfig, ChatLimitsConfig, CommonLimitsConfig,
    DemoAccountConfig, GrantAdminAccessConfig, MediaLimitsConfig, MinClientVersion,
    RemoteBotConfig,
};
use file_dynamic::ConfigFileDynamic;
use file_email_content::EmailContentFile;
use model::CustomReportsConfig;
pub use model::{ClientFeaturesConfig, ClientFeaturesConfigInternal};
use model_server_data::{AttributesFileInternal, ProfileAttributesInternal};
use sha2::{Digest, Sha256};
use simple_backend_config::{SimpleBackendConfig, file::SimpleBackendConfigFile};
use simple_backend_utils::IntoReportFromString;

use self::file::{ConfigFile, LocationConfig};
use crate::{
    file::{GeneralConfig, ProfileLimitsConfig},
    file_notification_content::NotificationContentFile,
};

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

    #[error("Invalid configuration")]
    InvalidConfiguration,
}

#[derive(Debug)]
pub struct ParsedFiles<'a> {
    pub server: &'a ConfigFile,
    pub dynamic: &'a ConfigFileDynamic,
    pub simple_backend: &'a SimpleBackendConfigFile,
    pub profile_attributes: Option<&'a AttributesFileInternal>,
    pub custom_reports: Option<&'a CustomReportsConfig>,
    pub client_features: Option<&'a ClientFeaturesConfig>,
    pub email_content: Option<&'a EmailContentFile>,
    pub notification_content: &'a NotificationContentFile,
    pub bot: Option<&'a BotConfigFile>,
}

#[derive(Debug)]
pub struct Config {
    file: ConfigFile,
    file_dynamic: ConfigFileDynamic,
    simple_backend_config: Arc<SimpleBackendConfig>,

    // Other configs
    mode: Option<AppMode>,
    profile_attributes: Option<ProfileAttributesInternal>,
    profile_attributes_sha256: Option<String>,
    custom_reports: Option<CustomReportsConfig>,
    custom_reports_sha256: Option<String>,
    client_features: Option<ClientFeaturesConfig>,
    client_features_sha256: Option<String>,
    email_content: Option<EmailContentFile>,
    notification_content: NotificationContentFile,

    profile_name_allowlist: ProfileNameAllowlistData,
    profile_name_regex: Option<Regex>,

    // Used only for config utils
    bot_config: Option<BotConfigFile>,
    profile_attributes_file: Option<AttributesFileInternal>,
}

impl Config {
    pub fn minimal_config_for_api_doc_json(
        simple_backend_config: Arc<SimpleBackendConfig>,
    ) -> Self {
        Self {
            file: ConfigFile::minimal_config_for_api_doc_json(),
            file_dynamic: ConfigFileDynamic::minimal_config_for_api_doc_json(),
            simple_backend_config,
            mode: None,
            profile_attributes: None,
            profile_attributes_sha256: None,
            custom_reports: None,
            custom_reports_sha256: None,
            client_features: None,
            client_features_sha256: None,
            email_content: None,
            notification_content: NotificationContentFile::default(),
            profile_name_allowlist: ProfileNameAllowlistData::default(),
            profile_name_regex: None,
            bot_config: None,
            profile_attributes_file: None,
        }
    }

    pub fn location(&self) -> LocationConfig {
        self.file.location.clone().unwrap_or_default()
    }

    /// Server should run in debug mode.
    ///
    /// Debug mode changes:
    /// * Routes for only related to benchmarking are available.
    /// * Axum JSON extractor shows errors.
    ///
    /// Check also [SimpleBackendConfig::debug_mode].
    pub fn debug_mode(&self) -> bool {
        self.simple_backend_config.debug_mode()
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

    pub fn remote_bot_login_allowed(&self) -> bool {
        self.file_dynamic
            .backend_config
            .remote_bot_login
            .unwrap_or_default()
    }

    pub fn local_admin_bot_enabled(&self) -> bool {
        self.file_dynamic
            .backend_config
            .local_bots
            .as_ref()
            .and_then(|v| v.admin)
            .unwrap_or_default()
    }

    pub fn local_user_bot_count(&self) -> u32 {
        self.file_dynamic
            .backend_config
            .local_bots
            .as_ref()
            .and_then(|v| v.users)
            .unwrap_or_default()
    }

    pub fn bot_config_file(&self) -> Option<&Path> {
        self.file.config_files.bot.as_deref()
    }

    pub fn limits_common(&self) -> CommonLimitsConfig {
        self.file
            .limits
            .as_ref()
            .and_then(|v| v.common.as_ref().cloned())
            .unwrap_or_default()
    }

    pub fn limits_account(&self) -> AccountLimitsConfig {
        self.file
            .limits
            .as_ref()
            .and_then(|v| v.account.as_ref().cloned())
            .unwrap_or_default()
    }

    pub fn limits_chat(&self) -> ChatLimitsConfig {
        self.file
            .limits
            .as_ref()
            .and_then(|v| v.chat.as_ref().cloned())
            .unwrap_or_default()
    }

    pub fn limits_media(&self) -> MediaLimitsConfig {
        self.file
            .limits
            .as_ref()
            .and_then(|v| v.media.as_ref().cloned())
            .unwrap_or_default()
    }

    pub fn limits_profile(&self) -> ProfileLimitsConfig {
        self.file
            .limits
            .as_ref()
            .and_then(|v| v.profile.as_ref().cloned())
            .unwrap_or_default()
    }

    pub fn profile_attributes(&self) -> Option<&ProfileAttributesInternal> {
        self.profile_attributes.as_ref()
    }

    pub fn profile_attributes_sha256(&self) -> Option<&str> {
        self.profile_attributes_sha256.as_deref()
    }

    pub fn custom_reports(&self) -> Option<&CustomReportsConfig> {
        self.custom_reports.as_ref()
    }

    pub fn custom_reports_sha256(&self) -> Option<&str> {
        self.custom_reports_sha256.as_deref()
    }

    pub fn client_features(&self) -> Option<&ClientFeaturesConfig> {
        self.client_features.as_ref()
    }

    pub fn client_features_sha256(&self) -> Option<&str> {
        self.client_features_sha256.as_deref()
    }

    pub fn email_content(&self) -> Option<&EmailContentFile> {
        self.email_content.as_ref()
    }

    pub fn notification_content(&self) -> &NotificationContentFile {
        &self.notification_content
    }

    pub fn demo_account_config(&self) -> Option<&Vec<DemoAccountConfig>> {
        self.file.demo_account.as_ref()
    }

    pub fn simple_backend(&self) -> &SimpleBackendConfig {
        &self.simple_backend_config
    }

    pub fn simple_backend_arc(&self) -> Arc<SimpleBackendConfig> {
        self.simple_backend_config.clone()
    }

    pub fn profile_name_allowlist(&self) -> &ProfileNameAllowlistData {
        &self.profile_name_allowlist
    }

    pub fn profile_name_regex(&self) -> Option<&Regex> {
        self.profile_name_regex.as_ref()
    }

    pub fn api_obfuscation_salt(&self) -> Option<&str> {
        self.file.api.obfuscation_salt.as_deref()
    }

    pub fn min_client_version(&self) -> Option<MinClientVersion> {
        self.file.api.min_client_version
    }

    pub fn remote_bots(&self) -> &[RemoteBotConfig] {
        &self.file.remote_bots
    }

    pub fn automatic_profile_search(&self) -> &AutomaticProfileSearchConfig {
        &self.file.automatic_profile_search
    }

    pub fn general(&self) -> &GeneralConfig {
        &self.file.general
    }

    pub fn parsed_files(&self) -> ParsedFiles {
        ParsedFiles {
            server: &self.file,
            dynamic: &self.file_dynamic,
            simple_backend: self.simple_backend().parsed_file(),
            profile_attributes: self.profile_attributes_file.as_ref(),
            custom_reports: self.custom_reports(),
            client_features: self.client_features(),
            email_content: self.email_content(),
            notification_content: self.notification_content(),
            bot: self.bot_config.as_ref(),
        }
    }
}

pub fn get_config(
    args_config: ArgsConfig,
    backend_code_version: String,
    backend_semver_version: String,
    save_default_config_if_not_found: bool,
) -> Result<Config, GetConfigError> {
    let simple_backend_config = simple_backend_config::get_config(
        args_config.server,
        backend_code_version,
        backend_semver_version,
        save_default_config_if_not_found,
    )
    .change_context(GetConfigError::SimpleBackendError)?;

    let file_config =
        file::ConfigFile::load_from_default_location(save_default_config_if_not_found)
            .change_context(GetConfigError::LoadFileError)?;

    let file_dynamic = ConfigFileDynamic::load_from_current_dir(save_default_config_if_not_found)
        .change_context(GetConfigError::LoadFileError)?;

    let (profile_attributes, profile_attributes_sha256, profile_attributes_file) =
        if let Some(path) = &file_config.config_files.profile_attributes {
            let attributes =
                std::fs::read_to_string(path).change_context(GetConfigError::LoadFileError)?;
            let mut profile_attributes_sha256 = Sha256::new();
            profile_attributes_sha256.update(attributes.as_bytes());
            let mut attributes_file: AttributesFileInternal =
                toml::from_str(&attributes).change_context(GetConfigError::InvalidConfiguration)?;
            AttributeValuesCsvLoader::load_if_needed(
                &mut attributes_file,
                &mut profile_attributes_sha256,
            )
            .change_context(GetConfigError::LoadFileError)?;
            let profile_attributes_sha256 = format!("{:x}", profile_attributes_sha256.finalize());
            let attributes = attributes_file
                .clone()
                .validate()
                .into_error_string(GetConfigError::InvalidConfiguration)?;
            (
                Some(attributes),
                Some(profile_attributes_sha256),
                Some(attributes_file),
            )
        } else {
            (None, None, None)
        };

    let (custom_reports, custom_reports_sha256) =
        if let Some(path) = &file_config.config_files.custom_reports {
            let custom_reports =
                std::fs::read_to_string(path).change_context(GetConfigError::LoadFileError)?;
            let custom_reports_sha256 = format!("{:x}", Sha256::digest(custom_reports.as_bytes()));
            let mut custom_reports: CustomReportsConfig = toml::from_str(&custom_reports)
                .change_context(GetConfigError::InvalidConfiguration)?;
            custom_reports
                .validate_and_sort_by_id()
                .into_error_string(GetConfigError::InvalidConfiguration)?;
            (Some(custom_reports), Some(custom_reports_sha256))
        } else {
            (None, None)
        };

    let (client_features, client_features_sha256) =
        if let Some(path) = &file_config.config_files.client_features {
            let features = std::fs::read_to_string(path)
                .change_context(GetConfigError::LoadFileError)
                .attach_printable_lazy(|| path.to_string_lossy().to_string())?;
            let sha256 = format!("{:x}", Sha256::digest(features.as_bytes()));
            let features: ClientFeaturesConfigInternal =
                toml::from_str(&features).change_context(GetConfigError::InvalidConfiguration)?;
            let features = features
                .to_client_features_config()
                .into_error_string(GetConfigError::InvalidConfiguration)?;
            (Some(features), Some(sha256))
        } else {
            (None, None)
        };

    let email_content = if let Some(path) = &file_config.config_files.email_content {
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

    let notification_content = if let Some(path) = &file_config.config_files.notification_content {
        NotificationContentFile::load(path).change_context(GetConfigError::LoadFileError)?
    } else {
        NotificationContentFile::default()
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

    let profile_name_regex = if let Some(regex) = client_features
        .as_ref()
        .and_then(|v| v.profile.profile_name_regex.as_ref())
    {
        let regex = Regex::new(regex).change_context(GetConfigError::LoadFileError)?;
        Some(regex)
    } else {
        None
    };

    let bot_config = if let Some(bot_config_file) = &file_config.config_files.bot {
        // Check that bot config file loads correctly
        let bot_config =
            BotConfigFile::load(bot_config_file).change_context(GetConfigError::LoadFileError)?;
        Some(bot_config)
    } else {
        None
    };

    let config = Config {
        simple_backend_config: simple_backend_config.into(),
        file: file_config,
        file_dynamic,
        mode: args_config.mode.clone(),
        profile_attributes,
        profile_attributes_sha256,
        custom_reports,
        custom_reports_sha256,
        client_features,
        client_features_sha256,
        email_content,
        notification_content,
        profile_name_allowlist,
        profile_name_regex,
        bot_config,
        profile_attributes_file,
    };

    Ok(config)
}
