use std::{
    num::NonZeroU8,
    path::{Path, PathBuf},
};

use error_stack::{Result, ResultExt};
// Re-export for test-mode crate
pub use model_server_data::EmailAddress;
use model_server_state::DemoModeId;
use serde::{Deserialize, Serialize};
use simple_backend_config::file::ConfigFileUtils;
use url::Url;
use utils::DurationValue;

pub mod utils;

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

# profile_attributes_file = "server_config_profile_attributes.toml"
# bot_config_file = "server_config_bots.toml"
# email_content_file = "server_config_email_content.toml"

# [grant_admin_access]
# email = "admin@example.com"

# [location]
# latitude_top_left = 70.1
# longitude_top_left = 19.5
# latitude_bottom_right = 59.8
# longitude_bottom_right = 31.58
# index_cell_square_km = 255       # 1-255 and area width and height must be larger than 255 km

# [external_services]
# account_internal = "http://127.0.0.1:4000"
# media_internal = "http://127.0.0.1:4000"

# [limits.account]
# account_deletion_wait_time_seconds = "90d"

# [limits.chat]
# like_limit_reset_time_utc_offset_hours = 0

# [limits.media]
# concurrent_content_uploads = 10
# max_content_count = 20
# unused_content_wait_time_seconds = "90d"

# [[profile_name_allowlist]]
# csv_file = "names.csv"
# delimiter = ";"
# column_index = 0
# start_row_index = 1

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
    pub bot_config_file: Option<PathBuf>,
    pub profile_attributes_file: Option<PathBuf>,
    pub email_content_file: Option<PathBuf>,

    pub api_obfuscation_salt: Option<String>,
    pub components: Option<Components>,
    pub grant_admin_access: Option<GrantAdminAccessConfig>,
    pub location: Option<LocationConfig>,
    pub external_services: Option<ExternalServices>,
    pub demo_mode: Option<Vec<DemoModeConfig>>,
    pub limits: Option<LimitsConfig>,
    pub profile_name_allowlist: Option<Vec<ProfiletNameAllowlistConfig>>,
}

impl ConfigFile {
    pub fn minimal_config_for_api_doc_json() -> Self {
        Self {
            bot_config_file: None,
            profile_attributes_file: None,
            email_content_file: None,
            api_obfuscation_salt: None,
            components: Some(Components::default()),
            grant_admin_access: None,
            location: None,
            external_services: None,
            demo_mode: None,
            limits: None,
            profile_name_allowlist: None,
        }
    }

    pub fn load(dir: impl AsRef<Path>) -> Result<ConfigFile, ConfigFileError> {
        let config_string =
            ConfigFileUtils::load_string(dir, CONFIG_FILE_NAME, DEFAULT_CONFIG_FILE_TEXT)
                .change_context(ConfigFileError::SimpleBackendError)?;
        toml::from_str(&config_string).change_context(ConfigFileError::LoadConfig)
    }
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

/// Base URLs for external services
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
            // TODO: Add validation to the values? And more location unit tests?
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
    pub account: Option<AccountLimitsConfig>,
    pub chat: Option<ChatLimitsConfig>,
    pub media: Option<MediaLimitsConfig>,
}

/// Account related limits config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccountLimitsConfig {
    pub account_deletion_wait_time_seconds: DurationValue,
}

impl Default for AccountLimitsConfig {
    fn default() -> Self {
        Self {
            account_deletion_wait_time_seconds: DurationValue::from_days(90),
        }
    }
}

/// Chat releated limits config
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ChatLimitsConfig {
    pub like_limit_reset_time_utc_offset_hours: i8,
}

/// Media related limits config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaLimitsConfig {
    /// Concurrent media content uploads. Processing of the media content
    /// will be done sequentially.
    pub concurrent_content_uploads: usize,
    pub max_content_count: u8,
    pub unused_content_wait_time_seconds: DurationValue,
}

impl Default for MediaLimitsConfig {
    fn default() -> Self {
        Self {
            concurrent_content_uploads: 10,
            max_content_count: 20,
            unused_content_wait_time_seconds: DurationValue::from_days(90),
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
    /// First step password for getting demo mode access token.
    pub password_stage0: String,
    /// Second step password for getting demo mode access token.
    /// If this is quessed wrong, these demo mode credentials will
    /// be locked untill server restarts.
    pub password_stage1: String,
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
