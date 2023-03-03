pub mod args;
pub mod file;

use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
    path::{Path, PathBuf},
};

use clap::{arg, command, value_parser};

use error_stack::{Report, Result, ResultExt};
use serde::Deserialize;

use crate::utils::IntoReportExt;

use self::{
    args::{ArgsConfig, ServerComponent},
    file::{Components, ConfigFile, ConfigFileError, SocketConfig},
};

pub const DATABASE_MESSAGE_CHANNEL_BUFFER: usize = 32;

#[derive(thiserror::Error, Debug)]
pub enum GetConfigError {
    #[error("Get working directory error")]
    GetWorkingDir,
    #[error("File loading failed")]
    LoadFileError,
    #[error("Load config file")]
    LoadConfig,
}

pub struct Config {
    file: ConfigFile,
    database: PathBuf,
}

impl Config {
    pub fn database_dir(&self) -> &Path {
        &self.database
    }

    pub fn components(&self) -> &Components {
        &self.file.components
    }

    pub fn socket(&self) -> &SocketConfig {
        &self.file.socket
    }

    /// Server should run in debug mode.
    ///
    /// Debug mode changes:
    /// * Swagger UI is enabled.
    /// * Internal API is available at same port as the public API.
    pub fn debug_mode(&self) -> bool {
        self.file.debug.unwrap_or(false)
    }
}

pub fn get_config() -> Result<Config, GetConfigError> {
    let current_dir = std::env::current_dir().into_error(GetConfigError::GetWorkingDir)?;
    let file_config =
        file::ConfigFile::load(current_dir).change_context(GetConfigError::LoadFileError)?;
    let args_config = args::get_config();

    let database = if let Some(database) = args_config.database_dir {
        database
    } else {
        file_config.database.dir.clone()
    };

    Ok(Config {
        file: file_config,
        database,
    })
}
