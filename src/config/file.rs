use std::{
    io::Write,
    net::SocketAddr,
    path::{Path, PathBuf}, num::{NonZeroU16, NonZeroU8},
};

use error_stack::{Report, Result, ResultExt};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::utils::IntoReportExt;

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

admin_email = "admin@example.com"

[location]
latitude_top_left = 70.1
longitude_top_left = 19.5
latitude_bottom_right = 59.8
longitude_bottom_right = 31.58
index_cell_square_km = 1

[socket]
public_api = "127.0.0.1:3000"
internal_api = "127.0.0.1:3001"

[database]
dir = "database"

[components]
account = true
profile = true
media = true

# [external_services]
# account_internal = "http://127.0.0.1:4000"
# media_internal = "http://127.0.0.1:4000"
"#;

#[derive(thiserror::Error, Debug)]
pub enum ConfigFileError {
    #[error("Save default")]
    SaveDefault,
    #[error("Not a directory")]
    NotDirectory,
    #[error("Load config file")]
    LoadConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pub debug: Option<bool>,
    pub admin_email: String,
    pub components: Components,
    pub database: DatabaseConfig,
    pub socket: SocketConfig,
    pub location: LocationConfig,
    pub external_services: Option<ExternalServices>,
}

impl ConfigFile {
    pub fn save_default(dir: impl AsRef<Path>) -> Result<(), ConfigFileError> {
        let file_path =
            Self::default_config_file_path(dir).change_context(ConfigFileError::SaveDefault)?;
        let mut file = std::fs::File::create(file_path).into_error(ConfigFileError::SaveDefault)?;
        file.write_all(DEFAULT_CONFIG_FILE_TEXT.as_bytes())
            .into_error(ConfigFileError::SaveDefault)?;
        Ok(())
    }

    pub fn load(dir: impl AsRef<Path>) -> Result<ConfigFile, ConfigFileError> {
        let file_path =
            Self::default_config_file_path(&dir).change_context(ConfigFileError::LoadConfig)?;
        if !file_path.exists() {
            Self::save_default(dir).change_context(ConfigFileError::LoadConfig)?;
        }

        let config_string =
            std::fs::read_to_string(file_path).into_error(ConfigFileError::LoadConfig)?;
        toml::from_str(&config_string).into_error(ConfigFileError::LoadConfig)
    }

    pub fn default_config_file_path(dir: impl AsRef<Path>) -> Result<PathBuf, ConfigFileError> {
        if !dir.as_ref().is_dir() {
            return Err(Report::new(ConfigFileError::NotDirectory));
        }
        let mut file_path = dir.as_ref().to_path_buf();
        file_path.push(CONFIG_FILE_NAME);
        return Ok(file_path);
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Components {
    pub account: bool,
    pub profile: bool,
    pub media: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub dir: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SocketConfig {
    pub public_api: SocketAddr,
    pub internal_api: SocketAddr,
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
    /// Index cell map size.
    pub index_cell_square_km: NonZeroU8,
    /// Minimun "y-axis" angle for bottom right corner of the location index.
    /// Index can in reality end further away.
    pub latitude_bottom_right: f64,
    /// Minimum "x-axis" angle for top left corner of the location index.
    /// Index can in reality end further away.
    pub longitude_bottom_right: f64,
}
