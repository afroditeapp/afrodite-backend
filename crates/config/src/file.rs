use std::{
    num::NonZeroU8,
    path::{Path, PathBuf},
};

use error_stack::{Result, ResultExt};
// Re-export for test-mode crate
pub use model_account::EmailAddress;
use model_account::GoogleAccountId;
use model_server_state::DemoModeId;
use serde::{Deserialize, Serialize};
use simple_backend_config::file::ConfigFileUtils;
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

# profile_attributes_file = "server_config_profile_attributes.toml"
# bot_config_file = "server_config_bots.toml"
# email_content_file = "server_config_email_content.toml"

[grant_admin_access]
email = "admin@example.com"
google_account_id = "TODO"

[components]
account = true
profile = true
media = true
chat = true

# [location]
# latitude_top_left = 70.1
# longitude_top_left = 19.5
# latitude_bottom_right = 59.8
# longitude_bottom_right = 31.58
# index_cell_square_km = 255       # 1-255 and area width and height must be larger than 255 km

# [internal_api]
# Enable login and register route for bots
# bot_login = false

# [external_services]
# account_internal = "http://127.0.0.1:4000"
# media_internal = "http://127.0.0.1:4000"

# [queue_limits]
# content_upload = 10

# [limits]
# like_limit_reset_time_utc_offset_hours = 0

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

    pub components: Components,
    pub grant_admin_access: Option<GrantAdminAccessConfig>,
    pub location: Option<LocationConfig>,
    pub external_services: Option<ExternalServices>,
    pub internal_api: Option<InternalApiConfig>,
    pub queue_limits: Option<QueueLimitsConfig>,
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
            components: Components::default(),
            grant_admin_access: None,
            location: None,
            external_services: None,
            internal_api: None,
            queue_limits: None,
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

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Components {
    pub account: bool,
    pub profile: bool,
    pub media: bool,
    pub chat: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GrantAdminAccessConfig {
    /// Grant admin access to every new account which matches with email and
    /// Google account ID. If only either is set, then only that must match.
    ///
    /// By default admin access is granted for once only.
    #[serde(default)]
    pub for_every_matching_new_account: bool,
    pub email: Option<EmailAddress>,
    pub google_account_id: Option<GoogleAccountId>,
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

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct InternalApiConfig {
    /// Enable register and login HTTP routes for bots through internal API socket.
    /// Note that debug option with this makes no authentication logins possible.
    pub bot_login: bool,
    /// Enable microservice mode related internal routes.
    #[serde(default)]
    pub microservice: bool,
}

/// Server queue limits
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QueueLimitsConfig {
    /// Simultaneous media content uploads. Processing of the media content
    /// will be done sequentially.
    ///
    /// Default: 10
    pub content_upload: usize,
}

impl Default for QueueLimitsConfig {
    fn default() -> Self {
        Self { content_upload: 10 }
    }
}

/// Limits config
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct LimitsConfig {
    pub like_limit_reset_time_utc_offset_hours: i8,
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
