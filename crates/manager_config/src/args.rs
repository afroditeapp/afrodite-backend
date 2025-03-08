//! Config given as command line arguments

use std::path::PathBuf;

use clap::{arg, command, Args, Parser};
use error_stack::{Result, ResultExt};
use manager_api::TlsConfig;
use manager_model::ManagerInstanceName;
use simple_backend_utils::ContextExt;
use url::Url;

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
    /// TLS root certificate for API client. If not present, value from
    /// current directory's config file is used.
    #[arg(short = 'c', long, value_name = "FILE")]
    pub tls_root_cert: Option<PathBuf>,
    /// TLS client authentication certificate for API client.
    /// If not present, value from current directory's config file is used.
    #[arg(long, value_name = "FILE")]
    pub tls_client_auth_cert: Option<PathBuf>,
    /// TLS client authentication certificate private key for API client.
    /// If not present, value from current directory's config file is used.
    #[arg(long, value_name = "FILE")]
    pub tls_client_auth_cert_private_key: Option<PathBuf>,
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

            Ok(file_config.manager.api_key)
        }
    }

    pub fn api_url(&self) -> Result<Url, GetConfigError> {
        if let Some(api_url) = self.api_url.clone() {
            return Ok(api_url);
        }

        let current_dir = std::env::current_dir().change_context(GetConfigError::GetWorkingDir)?;

        let file_config = super::file::ConfigFile::load_config(current_dir)
            .change_context(GetConfigError::LoadFileError)?;

        let url = if let Some(port) = file_config.socket.second_public_api_localhost_only_port {
            format!("tcp://localhost:{}", port)
        } else if let Some(addr) = file_config.socket.public_api {
            let scheme = if file_config.tls.is_some() {
                "tls"
            } else {
                "tcp"
            };
            format!("{}://localhost:{}", scheme, addr.port())
        } else {
            return Err(GetConfigError::LoadConfig.report())
                .attach_printable("No manager API server enabled from config file");
        };

        Url::parse(&url).change_context(GetConfigError::InvalidConstant)
    }

    pub fn tls_config(&self) -> Result<Option<TlsConfig>, GetConfigError> {
        let tls_arg_config = (self.tls_root_cert.clone(), self.tls_client_auth_cert.clone(), self.tls_client_auth_cert_private_key.clone());
        if let (Some(root), Some(client_auth), Some(client_auth_private_key)) = tls_arg_config {
            let config = TlsConfig::new(root, client_auth, client_auth_private_key)
                    .change_context(GetConfigError::ReadCertificateError)?;
            return Ok(Some(config));
        }

        let current_dir = std::env::current_dir().change_context(GetConfigError::GetWorkingDir)?;

        if ConfigFile::exists(&current_dir)
            .change_context(GetConfigError::CheckConfigFileExistanceError)?
        {
            let file_config =
                super::file::ConfigFile::save_default_if_not_exist_and_load(current_dir)
                    .change_context(GetConfigError::LoadFileError)?;

            if let Some(tls) = file_config.tls {
                let config = TlsConfig::new(
                    self.tls_root_cert.clone().unwrap_or(tls.root_cert),
                    self.tls_client_auth_cert.clone().unwrap_or(tls.public_api_cert),
                    self.tls_client_auth_cert_private_key.clone().unwrap_or(tls.public_api_key),
                )
                    .change_context(GetConfigError::ReadCertificateError)?;
                Ok(Some(config))
            } else {
                Ok(None)
            }
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

            Ok(file_config.manager.name)
        }
    }
}

#[derive(Parser, Debug, Clone)]
pub enum ApiCommand {
    AvailableInstances,
    EncryptionKey {
        encryption_key_name: String,
    },
    SystemInfo,
    SoftwareStatus,
    SoftwareDownload,
    SoftwareInstall {
        name: String,
        sha256: String,
    },
}
