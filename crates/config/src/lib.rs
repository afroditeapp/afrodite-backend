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
pub mod file_email_content;
pub mod file_notification_content;
pub mod file_web_content;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use bot_config_file::BotConfigFile;
use csv::{
    attribute_values::AttributeValuesCsvLoader,
    profile_name_allowlist::{ProfileNameAllowlistBuilder, ProfileNameAllowlistData},
};
use error_stack::{Result, ResultExt};
use file::{
    AccountLimitsConfig, AutomaticProfileSearchConfig, ChatLimitsConfig, CommonLimitsConfig,
    DemoAccountConfig, GrantAdminAccessConfig, MediaLimitsConfig, RemoteBotLoginConfig,
};
use file_email_content::EmailContentFile;
use file_web_content::WebContentFile;
pub use model::{
    AdminBotConfig, BackendConfig, ClientFeaturesConfig, ClientFeaturesConfigInternal,
};
use model::{CustomReportsConfig, ScheduledTasksConfig};
use model_server_data::{AttributesFileInternal, ProfileAttributesInternal};
use sha2::{Digest, Sha256};
use simple_backend_config::{SimpleBackendConfig, args::ServerMode, file::SimpleBackendConfigFile};
use simple_backend_utils::{
    IntoReportFromString, dir::abs_path_for_directory_or_file_which_might_not_exists,
};

use self::file::{ConfigFile, LocationConfig};
use crate::{
    file::{ClientVersionTrackingConfig, GeneralConfig, MinClientVersion, ProfileLimitsConfig},
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
    pub simple_backend: &'a SimpleBackendConfigFile,
    pub profile_attributes: &'a AttributesFileInternal,
    pub custom_reports: &'a CustomReportsConfig,
    pub client_features: &'a ClientFeaturesConfig,
    pub email_content: &'a EmailContentFile,
    pub notification_content: &'a NotificationContentFile,
    /// Might be Default implementation
    pub web_content: &'a WebContentFile,
    pub bot: &'a BotConfigFile,
}

#[derive(Debug)]
pub struct Config {
    file: ConfigFile,
    simple_backend_config: Arc<SimpleBackendConfig>,

    // Other configs
    profile_attributes: ProfileAttributesInternal,
    profile_attributes_sha256: String,
    custom_reports: CustomReportsConfig,
    custom_reports_sha256: String,
    client_features: ClientFeaturesConfig,
    client_features_sha256: String,
    client_features_internal: ClientFeaturesConfigInternal,
    email_content: EmailContentFile,
    notification_content: NotificationContentFile,
    web_content: WebContentFile,

    profile_name_allowlist: ProfileNameAllowlistData,
    profile_name_regex: Option<Regex>,

    bot_config_abs_file_path: PathBuf,

    // Used only for config utils
    bot_config: BotConfigFile,
    profile_attributes_file: AttributesFileInternal,
}

impl Config {
    pub fn minimal_config_for_api_doc_json(
        simple_backend_config: Arc<SimpleBackendConfig>,
    ) -> Self {
        Self {
            file: ConfigFile::minimal_config_for_api_doc_json(),
            simple_backend_config,
            profile_attributes: ProfileAttributesInternal::default(),
            profile_attributes_sha256: String::new(),
            custom_reports: CustomReportsConfig::default(),
            custom_reports_sha256: String::new(),
            client_features: ClientFeaturesConfig::default(),
            client_features_sha256: String::new(),
            client_features_internal: ClientFeaturesConfigInternal::default(),
            email_content: EmailContentFile::default(),
            notification_content: NotificationContentFile::default(),
            web_content: WebContentFile::default(),
            profile_name_allowlist: ProfileNameAllowlistData::default(),
            profile_name_regex: None,
            bot_config_abs_file_path: PathBuf::new(),
            bot_config: BotConfigFile::default(),
            profile_attributes_file: AttributesFileInternal::default(),
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

    pub fn grant_admin_access_config(&self) -> Option<&GrantAdminAccessConfig> {
        self.file.grant_admin_access.as_ref()
    }

    pub fn bot_config_abs_file_path(&self) -> &Path {
        &self.bot_config_abs_file_path
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

    pub fn profile_attributes(&self) -> &ProfileAttributesInternal {
        &self.profile_attributes
    }

    pub fn profile_attributes_sha256(&self) -> &str {
        &self.profile_attributes_sha256
    }

    pub fn custom_reports(&self) -> &CustomReportsConfig {
        &self.custom_reports
    }

    pub fn custom_reports_sha256(&self) -> &str {
        &self.custom_reports_sha256
    }

    pub fn client_features(&self) -> &ClientFeaturesConfig {
        &self.client_features
    }

    pub fn client_features_internal(&self) -> &ClientFeaturesConfigInternal {
        &self.client_features_internal
    }

    pub fn client_features_sha256(&self) -> &str {
        &self.client_features_sha256
    }

    pub fn scheduled_tasks(&self) -> ScheduledTasksConfig {
        self.client_features_internal().scheduled_tasks()
    }

    pub fn email_content(&self) -> &EmailContentFile {
        &self.email_content
    }

    pub fn notification_content(&self) -> &NotificationContentFile {
        &self.notification_content
    }

    pub fn web_content(&self) -> &WebContentFile {
        &self.web_content
    }

    pub fn demo_account_config(&self) -> Option<&Vec<DemoAccountConfig>> {
        self.file.demo_accounts.as_ref()
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

    pub fn client_version_tracking(&self) -> Option<&ClientVersionTrackingConfig> {
        self.file.api.client_version_tracking.as_ref()
    }

    pub fn remote_bot_login(&self) -> &RemoteBotLoginConfig {
        &self.file.remote_bot_login
    }

    pub fn automatic_profile_search(&self) -> &AutomaticProfileSearchConfig {
        &self.file.automatic_profile_search
    }

    pub fn general(&self) -> &GeneralConfig {
        &self.file.general
    }

    pub fn parsed_files(&self) -> ParsedFiles<'_> {
        ParsedFiles {
            server: &self.file,
            simple_backend: self.simple_backend().parsed_file(),
            profile_attributes: &self.profile_attributes_file,
            custom_reports: self.custom_reports(),
            client_features: self.client_features(),
            email_content: self.email_content(),
            notification_content: self.notification_content(),
            web_content: self.web_content(),
            bot: &self.bot_config,
        }
    }
}

/// Changes working directory to config file directory
pub fn get_config(
    args_config: ServerMode,
    backend_code_version: String,
    backend_semver_version: String,
    save_default_config_if_not_found: bool,
) -> Result<Config, GetConfigError> {
    let simple_backend_config = simple_backend_config::get_config(
        args_config,
        backend_code_version,
        backend_semver_version,
        save_default_config_if_not_found,
    )
    .change_context(GetConfigError::SimpleBackendError)?;

    let file_config =
        file::ConfigFile::load_from_default_location(save_default_config_if_not_found)
            .change_context(GetConfigError::LoadFileError)?;

    let (profile_attributes, profile_attributes_sha256, profile_attributes_file) = {
        let path = Path::new(AttributesFileInternal::CONFIG_FILE_NAME);
        if !path.exists() && save_default_config_if_not_found {
            std::fs::write(path, AttributesFileInternal::DEFAULT_CONFIG_FILE_TEXT)
                .change_context(GetConfigError::LoadFileError)?;
        }
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
        (attributes, profile_attributes_sha256, attributes_file)
    };

    let (custom_reports, custom_reports_sha256) = {
        let path = Path::new(CustomReportsConfig::CONFIG_FILE_NAME);
        if !path.exists() && save_default_config_if_not_found {
            std::fs::write(path, CustomReportsConfig::DEFAULT_CONFIG_FILE_TEXT)
                .change_context(GetConfigError::LoadFileError)?;
        }
        let custom_reports =
            std::fs::read_to_string(path).change_context(GetConfigError::LoadFileError)?;
        let custom_reports_sha256 = format!("{:x}", Sha256::digest(custom_reports.as_bytes()));
        let mut custom_reports: CustomReportsConfig =
            toml::from_str(&custom_reports).change_context(GetConfigError::InvalidConfiguration)?;
        custom_reports
            .validate_and_sort_by_id()
            .into_error_string(GetConfigError::InvalidConfiguration)?;
        (custom_reports, custom_reports_sha256)
    };

    let (client_features, client_features_sha256, client_features_internal) = {
        let path = Path::new(ClientFeaturesConfigInternal::CONFIG_FILE_NAME);
        if !path.exists() && save_default_config_if_not_found {
            std::fs::write(path, ClientFeaturesConfigInternal::DEFAULT_CONFIG_FILE_TEXT)
                .change_context(GetConfigError::LoadFileError)?;
        }
        let features = std::fs::read_to_string(path)
            .change_context(GetConfigError::LoadFileError)
            .attach_printable_lazy(|| path.to_string_lossy().to_string())?;
        let sha256 = format!("{:x}", Sha256::digest(features.as_bytes()));
        let features_internal: ClientFeaturesConfigInternal =
            toml::from_str(&features).change_context(GetConfigError::InvalidConfiguration)?;
        let features = features_internal
            .clone()
            .to_client_features_config()
            .into_error_string(GetConfigError::InvalidConfiguration)?;
        (features, sha256, features_internal)
    };

    let email_content = EmailContentFile::load(
        EmailContentFile::CONFIG_FILE_NAME,
        save_default_config_if_not_found,
    )
    .change_context(GetConfigError::LoadFileError)?;

    let notification_content = NotificationContentFile::load(
        NotificationContentFile::CONFIG_FILE_NAME,
        save_default_config_if_not_found,
    )
    .change_context(GetConfigError::LoadFileError)?;

    let web_content = WebContentFile::load(
        Path::new(WebContentFile::CONFIG_FILE_NAME),
        save_default_config_if_not_found,
    )
    .change_context(GetConfigError::LoadFileError)?;

    let mut allowlist_builder = ProfileNameAllowlistBuilder::default();
    let csv_configs = file_config
        .profile_name_allowlists
        .as_ref()
        .map(|v| v.iter())
        .unwrap_or_default();
    for c in csv_configs {
        allowlist_builder
            .load(c)
            .change_context(GetConfigError::ProfileNameAllowlistError)?;
    }
    let profile_name_allowlist = allowlist_builder.build();

    let profile_name_regex =
        if let Some(regex) = client_features_internal.profile.profile_name_regex.as_ref() {
            let regex = Regex::new(regex).change_context(GetConfigError::LoadFileError)?;
            Some(regex)
        } else {
            None
        };

    let bot_config_abs_file_path =
        abs_path_for_directory_or_file_which_might_not_exists(BotConfigFile::CONFIG_FILE_NAME)
            .change_context(GetConfigError::LoadFileError)?;
    let bot_config =
        BotConfigFile::load(&bot_config_abs_file_path, save_default_config_if_not_found)
            .change_context(GetConfigError::LoadFileError)?;

    let config = Config {
        simple_backend_config: simple_backend_config.into(),
        file: file_config,
        profile_attributes,
        profile_attributes_sha256,
        custom_reports,
        custom_reports_sha256,
        client_features,
        client_features_sha256,
        client_features_internal,
        email_content,
        notification_content,
        web_content,
        profile_name_allowlist,
        profile_name_regex,
        bot_config_abs_file_path,
        bot_config,
        profile_attributes_file,
    };

    Ok(config)
}
