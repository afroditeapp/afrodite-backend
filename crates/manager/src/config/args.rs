//! Config given as command line arguments

use std::path::PathBuf;

use clap::{arg, command, Args, Parser};
use error_stack::{Result, ResultExt};
use manager_model::{ManagerInstanceName, SoftwareOptions};
use tokio_rustls::rustls::RootCertStore;
use url::Url;

use crate::api::client::ManagerClient;

use super::{file::ConfigFile, GetConfigError};

#[derive(Args, Debug, Clone)]
pub struct ManagerApiClientMode {
    /// API key for accessing the manager API. If not present, value from
    /// current directory's config file is used.
    #[arg(short = 'k', long, value_name = "KEY")]
    api_key: Option<String>,
    /// API URL for accessing the manager API. If not present, value from
    /// current directory's config file is used.
    #[arg(short = 'u', long, value_name = "URL")]
    pub api_url: Option<Url>,
    /// Root certificate for API client. If not present, value from
    /// current directory's config file is used.
    #[arg(short = 'c', long, value_name = "FILE")]
    pub root_certificate: Option<PathBuf>,
    /// Name of the manager instance which receives the API request. If not
    /// present, value from current directory's config file is used.
    #[arg(short = 'n', long, value_name = "NAME")]
    pub name: Option<String>,

    #[command(subcommand)]
    pub api_command: ApiCommand,
}

impl ManagerApiClientMode {
    pub fn api_key(&self) -> Result<String, GetConfigError> {
        if let Some(api_key) = self.api_key.clone() {
            Ok(api_key)
        } else {
            let current_dir =
                std::env::current_dir().change_context(GetConfigError::GetWorkingDir)?;
            let file_config = ConfigFile::load_config(current_dir)
                .change_context(GetConfigError::LoadFileError)?;

            Ok(file_config.api_key)
        }
    }

    pub fn api_url(&self) -> Result<Url, GetConfigError> {
        if let Some(api_url) = self.api_url.clone() {
            return Ok(api_url);
        }

        let current_dir = std::env::current_dir().change_context(GetConfigError::GetWorkingDir)?;

        let file_config = super::file::ConfigFile::load_config(current_dir)
            .change_context(GetConfigError::LoadFileError)?;

        let scheme = if file_config.tls.is_some() {
            "tls"
        } else {
            "tcp"
        };

        let url = format!("{}://localhost:{}", scheme, file_config.socket.public_api.port());

        Url::parse(&url).change_context(GetConfigError::InvalidConstant)
    }

    fn root_certificate_file(&self) -> Result<Option<PathBuf>, GetConfigError> {
        if let Some(root_certificate) = self.root_certificate.clone() {
            return Ok(Some(root_certificate));
        }

        let current_dir = std::env::current_dir().change_context(GetConfigError::GetWorkingDir)?;

        if ConfigFile::exists(&current_dir)
            .change_context(GetConfigError::CheckConfigFileExistanceError)?
        {
            let file_config =
                super::file::ConfigFile::save_default_if_not_exist_and_load(current_dir)
                    .change_context(GetConfigError::LoadFileError)?;

            Ok(file_config.tls.map(|v| v.root_certificate))
        } else {
            Ok(None)
        }
    }

    pub fn root_certificate(&self) -> Result<Option<RootCertStore>, GetConfigError> {
        if let Some(root_certificate_file) = self.root_certificate_file()? {
            let cert = ManagerClient::load_root_certificate(root_certificate_file)
                .change_context(GetConfigError::ReadCertificateError)?;
            Ok(Some(cert))
        } else {
            Ok(None)
        }
    }

    pub fn manager_name(&self) -> Result<ManagerInstanceName, GetConfigError> {
        if let Some(name) = self.name.clone() {
            Ok(ManagerInstanceName::new(name))
        } else {
            let current_dir =
                std::env::current_dir().change_context(GetConfigError::GetWorkingDir)?;
            let file_config = ConfigFile::load_config(current_dir)
                .change_context(GetConfigError::LoadFileError)?;

            Ok(file_config.manager_name)
        }
    }
}

#[derive(Parser, Debug, Clone)]
pub enum ApiCommand {
    AvailableInstances,
    EncryptionKey {
        encryption_key_name: String,
    },
    LatestBuildInfo {
        #[arg(value_enum)]
        software: SoftwareOptions,
    },
    RequestUpdateSoftware {
        #[arg(value_enum)]
        software: SoftwareOptions,
        #[arg(short, long)]
        reboot: bool,
        #[arg(long)]
        reset_data: bool,
    },
    RequestRestartBackend {
        #[arg(long)]
        reset_data: bool,
    },
    SystemInfo,
    SoftwareInfo,
}
