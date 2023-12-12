use std::{
    num::NonZeroU8,
    path::{Path, PathBuf},
};

use error_stack::{Result, ResultExt};
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

# Also google account ID is required if sign in with google is enabled.
admin_email = "admin@example.com"

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
# image_upload = 10

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
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pub admin_email: String,
    pub components: Components,
    pub location: Option<LocationConfig>,
    pub bots: Option<StaticBotConfig>,
    pub external_services: Option<ExternalServices>,

    pub internal_api: Option<InternalApiConfig>,
    pub queue_limits: Option<QueueLimitsConfig>,
}

impl ConfigFile {
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
pub struct StaticBotConfig {
    pub man_image_dir: Option<PathBuf>,
    pub woman_image_dir: Option<PathBuf>,
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
}

/// Server queue limits
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QueueLimitsConfig {
    pub image_upload: usize,
}

impl Default for QueueLimitsConfig {
    fn default() -> Self {
        Self { image_upload: 10 }
    }
}
