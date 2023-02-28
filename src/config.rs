pub mod file;
pub mod args;

use std::{path::{PathBuf, Path}, convert::{TryFrom, TryInto}, collections::HashSet};

use clap::{arg, command, value_parser};

use error_stack::{Result, ResultExt, Report};
use serde::Deserialize;


use crate::utils::IntoReportExt;

use self::{file::{ConfigFileError, ConfigFile, Components}, args::{ArgsConfig, ServerComponent}};

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
}

pub fn get_config() -> Result<Config, GetConfigError> {
    let current_dir = std::env::current_dir()
        .into_error(GetConfigError::GetWorkingDir)?;
    let file_config =
        file::ConfigFile::load(current_dir)
            .change_context(GetConfigError::LoadFileError)?;
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
