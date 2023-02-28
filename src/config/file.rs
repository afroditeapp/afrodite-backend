use std::{path::{Path, PathBuf}, io::Write};

use error_stack::{Result, ResultExt, Report};
use serde::Deserialize;

use crate::{utils::IntoReportExt, server::database::git::file};

pub const CONFIG_FILE_NAME: &str = "server_config.toml";

pub const DEFAULT_CONFIG_FILE_TEXT: &str = r#"

[database]
dir = "database"

[components]
login = true
core = true
media = true


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


#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub components: Components,
    pub database: DatabaseConfig,
}

impl ConfigFile {
    pub fn save_default(dir: impl AsRef<Path>) -> Result<(), ConfigFileError> {
        let file_path = Self::default_config_file_path(dir)
            .change_context(ConfigFileError::SaveDefault)?;
        let mut file = std::fs::File::create(file_path)
            .into_error(ConfigFileError::SaveDefault)?;
        file.write_all(DEFAULT_CONFIG_FILE_TEXT.as_bytes())
            .into_error(ConfigFileError::SaveDefault)?;
        Ok(())
    }

    pub fn load(dir: impl AsRef<Path>) -> Result<ConfigFile, ConfigFileError> {
        let file_path = Self::default_config_file_path(&dir)
            .change_context(ConfigFileError::LoadConfig)?;
        if !file_path.exists() {
            Self::save_default(dir)
                .change_context(ConfigFileError::LoadConfig)?;
        }

        let config_string = std::fs::read_to_string(file_path)
            .into_error(ConfigFileError::LoadConfig)?;
        toml::from_str(&config_string)
            .into_error(ConfigFileError::LoadConfig)
    }

    pub fn default_config_file_path(dir: impl AsRef<Path>) -> Result<PathBuf, ConfigFileError> {
        if !dir.as_ref().is_dir() {
            return Err(Report::new(ConfigFileError::NotDirectory));
        }
        let mut file_path = dir.as_ref().to_path_buf();
        file_path.push(CONFIG_FILE_NAME);
        return Ok(file_path)
    }
}

#[derive(Debug, Deserialize)]
pub struct Components {
    pub login: bool,
    pub core: bool,
    pub media: bool,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub dir: PathBuf,
}
