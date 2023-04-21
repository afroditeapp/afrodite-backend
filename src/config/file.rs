use std::{
    io::Write,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use error_stack::{Report, Result, ResultExt};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::utils::IntoReportExt;

pub const CONFIG_FILE_NAME: &str = "server_config.toml";

pub const DEFAULT_CONFIG_FILE_TEXT: &str = r#"

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
    pub components: Components,
    pub database: DatabaseConfig,
    pub socket: SocketConfig,
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
