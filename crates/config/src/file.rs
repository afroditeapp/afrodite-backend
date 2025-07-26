use std::{
    collections::HashSet,
    num::NonZeroU8,
    path::{Path, PathBuf},
};

use error_stack::{Result, ResultExt};
use model::{AccountId, ClientVersion};
// Re-export for test-mode crate
pub use model_server_data::EmailAddress;
use model_server_state::DemoModeId;
use serde::{Deserialize, Serialize};
use simple_backend_config::file::ConfigFileUtils;
use simple_backend_utils::{
    ContextExt,
    time::{DurationValue, TimeValue, UtcTimeValue},
};
use url::Url;

// Kilpisjärvi ja Nuorgam
// latitude_top_left = 70.1
// longitude_top_left = 20.5
//
// Eckerö (Ahvenanmaa) ja Nuorgam
// latitude_top_left = 70.1
// longitude_top_left = 19.5

// Suomen eteläisin kärki (Hanko) ja Suomen itäisin piste
// latitude_bottom_right = 59.8
// longitude_bottom_right = 31.58

pub const CONFIG_FILE_NAME: &str = "server_config.toml";

pub const DEFAULT_CONFIG_FILE_TEXT: &str = r#"

# [config_files]
# bot = "server_config_bots.toml"
# email_content = "server_config_email_content.toml"
# notification_content = "server_config_notification_content.toml"
# profile_attributes = "server_config_profile_attributes.toml"
# custom_reports = "server_config_custom_reports.toml"
# client_features = "server_config_client_features.toml"

# [grant_admin_access]
# email = "admin@example.com"

# [location]
# latitude_top_left = 70.1
# longitude_top_left = 19.5
# latitude_bottom_right = 59.8
# longitude_bottom_right = 31.58
# index_cell_square_km = 255       # 1-255 and area width and height must be larger than 255 km

# [limits.common.processed_report_deletion_wait_duration]
# profile_name = "90d"
# profile_text = "90d"

# [limits.account]
# account_deletion_wait_duration = "90d"

# [limits.chat]
# max_public_key_count = 100

# [limits.media]
# concurrent_content_uploads = 10
# max_content_count = 20
# unused_content_wait_duration = "90d"

# [[profile_name_allowlist]]
# csv_file = "names.csv"
# delimiter = ";"
# column_index = 0
# start_row_index = 1

# [[remote_bot]]
# account_id = "TODO"
# password = "TODO"

# [automatic_profile_search]
# daily_start_time = "9:00"
# daily_end_time = "21:00"

"#;

#[derive(thiserror::Error, Debug)]
pub enum ConfigFileError {
    #[error("Simple backend error")]
    SimpleBackendError,

    #[error("Load config file")]
    LoadConfig,
    #[error("Editing config file failed")]
    EditConfig,
    #[error("Saving edited config file failed")]
    SaveEditedConfig,

    #[error("Invalid config")]
    InvalidConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub config_files: ConfigFileConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub automatic_profile_search: AutomaticProfileSearchConfig,

    pub components: Option<Components>,
    pub grant_admin_access: Option<GrantAdminAccessConfig>,
    pub location: Option<LocationConfig>,
    pub external_services: Option<ExternalServices>,
    pub demo_mode: Option<Vec<DemoModeConfig>>,
    pub limits: Option<LimitsConfig>,
    pub profile_name_allowlist: Option<Vec<ProfiletNameAllowlistConfig>>,
    pub remote_bot: Option<Vec<RemoteBotConfig>>,
}

impl ConfigFile {
    pub fn minimal_config_for_api_doc_json() -> Self {
        Self {
            config_files: ConfigFileConfig::default(),
            api: ApiConfig::default(),
            automatic_profile_search: AutomaticProfileSearchConfig::default(),
            components: Some(Components::default()),
            grant_admin_access: None,
            location: None,
            external_services: None,
            demo_mode: None,
            limits: None,
            profile_name_allowlist: None,
            remote_bot: None,
        }
    }

    pub fn load(
        dir: impl AsRef<Path>,
        save_default_if_not_found: bool,
    ) -> Result<ConfigFile, ConfigFileError> {
        let config_string = ConfigFileUtils::load_string(
            dir,
            CONFIG_FILE_NAME,
            DEFAULT_CONFIG_FILE_TEXT,
            save_default_if_not_found,
        )
        .change_context(ConfigFileError::SimpleBackendError)?;
        let file: ConfigFile =
            toml::from_str(&config_string).change_context(ConfigFileError::LoadConfig)?;

        if let Some(remote_bots) = &file.remote_bot {
            let mut set = HashSet::<AccountId>::new();

            for b in remote_bots {
                let aid = b.account_id();
                if set.contains(&aid) {
                    return Err(ConfigFileError::InvalidConfig.report()).attach_printable(format!(
                        "Duplicate remote bot config for account {aid}"
                    ));
                }
                set.insert(aid);
            }
        }

        Ok(file)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ApiConfig {
    pub obfuscation_salt: Option<String>,
    pub min_client_version: Option<MinClientVersion>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConfigFileConfig {
    pub bot: Option<PathBuf>,
    pub email_content: Option<PathBuf>,
    pub notification_content: Option<PathBuf>,
    pub profile_attributes: Option<PathBuf>,
    pub custom_reports: Option<PathBuf>,
    pub client_features: Option<PathBuf>,
}

/// Enabled server components
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
pub struct Components {
    pub account: bool,
    pub profile: bool,
    pub media: bool,
    pub chat: bool,
}

impl Components {
    pub fn all_enabled() -> Self {
        Self {
            account: true,
            profile: true,
            media: true,
            chat: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GrantAdminAccessConfig {
    /// Grant admin access to every new account which matches with email.
    ///
    /// By default admin access is granted for once only.
    #[serde(default)]
    pub debug_for_every_matching_new_account: bool,
    /// Change matching to check only email domain.
    #[serde(default)]
    pub debug_match_only_email_domain: bool,
    pub email: EmailAddress,
}

/// Base URLs for defining server to server connections.
/// Only used in microservice mode.
#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct ExternalServices {
    pub account_internal: Option<Url>,
    pub media_internal: Option<Url>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LocationConfig {
    /// "y-axis" angle for top left corner of the location index.
    pub latitude_top_left: f64,
    /// "x-axis" angle for top left corner of the location index.
    pub longitude_top_left: f64,
    /// Minimun "y-axis" angle for bottom right corner of the location index.
    /// Index can in reality end further away.
    pub latitude_bottom_right: f64,
    /// Minimum "x-axis" angle for top left corner of the location index.
    /// Index can in reality end further away.
    pub longitude_bottom_right: f64,
    /// Index cell map size target value. Might be smaller or larger depending
    /// the supported tile sizes.
    pub index_cell_square_km: NonZeroU8,
}

impl Default for LocationConfig {
    fn default() -> Self {
        Self {
            // Just use Finland as default as that is tested to work.
            latitude_top_left: 70.1,
            longitude_top_left: 19.5,
            latitude_bottom_right: 59.8,
            longitude_bottom_right: 31.58,
            // Make matrix cells 255 square kilometers, so the matrix will not
            // consume that much of memory.
            index_cell_square_km: NonZeroU8::MAX,
        }
    }
}

/// Limits config
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct LimitsConfig {
    pub common: Option<CommonLimitsConfig>,
    pub account: Option<AccountLimitsConfig>,
    pub chat: Option<ChatLimitsConfig>,
    pub media: Option<MediaLimitsConfig>,
    pub profile: Option<ProfileLimitsConfig>,
}

/// Common limits config for all server components
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct CommonLimitsConfig {
    pub processed_report_deletion_wait_duration: ProcessedReportDeletionConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProcessedReportDeletionConfig {
    pub profile_name: DurationValue,
    pub profile_text: DurationValue,
}

impl Default for ProcessedReportDeletionConfig {
    fn default() -> Self {
        Self {
            profile_name: DurationValue::from_days(90),
            profile_text: DurationValue::from_days(90),
        }
    }
}

/// Account related limits config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccountLimitsConfig {
    pub inactivity_logout_wait_duration: DurationValue,
    pub account_deletion_wait_duration: DurationValue,
    pub init_deletion_for_inactive_accounts_wait_duration: DurationValue,
}

impl Default for AccountLimitsConfig {
    fn default() -> Self {
        Self {
            inactivity_logout_wait_duration: DurationValue::from_days(365),
            account_deletion_wait_duration: DurationValue::from_days(90),
            init_deletion_for_inactive_accounts_wait_duration: DurationValue::from_days(365 * 2), // About 2 years
        }
    }
}

/// Chat releated limits config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatLimitsConfig {
    pub max_public_key_count: u16,
    pub new_message_email_with_fcm_token: DurationValue,
    pub new_message_email_without_fcm_token: DurationValue,
}

impl Default for ChatLimitsConfig {
    fn default() -> Self {
        Self {
            max_public_key_count: 100,
            new_message_email_with_fcm_token: DurationValue::from_days(7),
            new_message_email_without_fcm_token: DurationValue::from_days(1),
        }
    }
}

/// Media related limits config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaLimitsConfig {
    /// Concurrent media content uploads. Processing of the media content
    /// will be done sequentially.
    pub concurrent_content_uploads: usize,
    pub max_content_count: u8,
    pub unused_content_wait_duration: DurationValue,
}

impl Default for MediaLimitsConfig {
    fn default() -> Self {
        Self {
            concurrent_content_uploads: 10,
            max_content_count: 20,
            unused_content_wait_duration: DurationValue::from_days(90),
        }
    }
}

/// Profile related limits config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProfileLimitsConfig {
    pub profile_iterator_reset_daily_max_count: u16,
    pub profile_iterator_next_page_daily_max_count: u16,
}

impl Default for ProfileLimitsConfig {
    fn default() -> Self {
        Self {
            profile_iterator_reset_daily_max_count: 100,
            profile_iterator_next_page_daily_max_count: 1000,
        }
    }
}

/// Demo mode configuration.
///
/// Adding one or more demo mode configurations
/// will enable demo mode HTTP routes.
///
/// WARNING: This gives access to all/specific accounts.
#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct DemoModeConfig {
    pub database_id: DemoModeId,
    pub username: String,
    /// If this is quessed wrong, these demo mode credentials will
    /// be locked until server restarts.
    pub password: String,
    /// If true then all accounts are accessible.
    /// Overrides `accessible_accounts`.
    #[serde(default)]
    pub access_all_accounts: bool,
    /// AccountIds for accounts that are accessible in demo mode.
    #[serde(default)]
    pub accessible_accounts: Vec<simple_backend_utils::UuidBase64Url>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProfiletNameAllowlistConfig {
    pub csv_file: PathBuf,
    pub delimiter: char,
    /// Column index starting from zero.
    pub column_index: usize,
    /// Index for first row where data reading starts. The index values
    /// starts from zero.
    pub start_row_index: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(try_from = "String")]
pub struct MinClientVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl MinClientVersion {
    pub fn received_version_is_accepted(&self, received: ClientVersion) -> bool {
        if received.major > self.major {
            true
        } else if received.major < self.major {
            false
        } else if received.minor > self.minor {
            true
        } else if received.minor < self.minor {
            false
        } else if received.patch > self.patch {
            true
        } else if received.patch < self.patch {
            false
        } else {
            // Versions are equal
            true
        }
    }
}

impl TryFrom<String> for MinClientVersion {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let mut numbers = value.split('.');
        let error = || format!("Version {value} is not formatted like 1.0.0");
        let major_str = numbers.next().ok_or_else(error)?;
        let minor_str = numbers.next().ok_or_else(error)?;
        let patch_str = numbers.next().ok_or_else(error)?;

        let major = major_str.parse::<u16>().map_err(|e| e.to_string())?;
        let minor = minor_str.parse::<u16>().map_err(|e| e.to_string())?;
        let patch = patch_str.parse::<u16>().map_err(|e| e.to_string())?;

        Ok(MinClientVersion {
            major,
            minor,
            patch,
        })
    }
}

/// Remote bot config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RemoteBotConfig {
    account_id: simple_backend_utils::UuidBase64UrlToml,
    password: String,
}

impl RemoteBotConfig {
    pub fn account_id(&self) -> AccountId {
        AccountId {
            aid: self.account_id.into(),
        }
    }

    pub fn password(&self) -> String {
        self.password.clone()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AutomaticProfileSearchConfig {
    pub daily_start_time: UtcTimeValue,
    pub daily_end_time: UtcTimeValue,
}

impl Default for AutomaticProfileSearchConfig {
    fn default() -> Self {
        const DEFAULT_START_TIME: TimeValue = TimeValue::new(9, 0);
        const DEFAULT_END_TIME: TimeValue = TimeValue::new(21, 0);

        Self {
            daily_start_time: UtcTimeValue(DEFAULT_START_TIME),
            daily_end_time: UtcTimeValue(DEFAULT_END_TIME),
        }
    }
}
