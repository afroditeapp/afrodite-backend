use std::{
    io::Write,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use error_stack::{Report, Result, ResultExt};
use manager_model::{ManagerInstanceName, SecureStorageEncryptionKey};
use serde::{Deserialize, Serialize};
use simple_backend_utils::time::UtcTimeValue;
use url::Url;

use super::GetConfigError;

pub const CONFIG_FILE_NAME: &str = "manager_config.toml";

pub const DEFAULT_CONFIG_FILE_TEXT: &str = r#"

# Required
# manager_name = "default"
# api_key = "password"
# scripts_dir = "/app-server-tools/manager-tools"
# storage_dir = "/app-secure-storage/app/app-manager-storage"

# log_timestamp = true # optional

[socket]
public_api = "127.0.0.1:4000"
# Second API has no TLS even if it is configured
# second_public_api_localhost_only_port = 4001

# [[remote_manager]]
# manager_name = "backup"
# url = "tls://127.0.0.1:4000"

# [secure_storage]
# key_storage_manager_name = "default"
# availability_check_path = "/app-secure-storage/app"
# -------- Optional --------
# Fall back to local encryption key if the manager instance is not available.
# Should not be used in production.
# encryption_key_text = ""

# [[server_encryption_key]]
# manager_name = "default"
# key_path = "data-key.key"

# [software_update_provider]
# manager_base_url = "tls://127.0.0.1:4000"
# backend_install_location = "/app-secure-storage/app/binaries/app-backend"
# backend_data_reset_dir = "/path/to/backend/data" # Optional

# [reboot_if_needed]
# time = "12:00"

# [system_info]
# log_services = ["afrodite-manager", "afrodite-backend"]

# [tls]
# public_api_cert = "tls/server.crt"
# public_api_key = "tls/server.key"
# root_certificate = "tls/root.crt"
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
    /// API key for manager API. All managers instances must use the same key.
    ///
    /// If the key is wrong the API access is denied untill manager is restarted.
    pub api_key: String,
    /// Directory for build and update files.
    pub storage_dir: PathBuf,
    pub scripts_dir: PathBuf,
    pub manager_name: ManagerInstanceName,
    pub socket: SocketConfig,

    // Optional configs
    #[serde(default)]
    pub remote_manager: Vec<ManagerInstance>,
    #[serde(default)]
    pub server_encryption_key: Vec<ServerEncryptionKey>,
    pub secure_storage: Option<SecureStorageConfig>,
    pub reboot_if_needed: Option<RebootIfNeededConfig>,
    pub software_update_provider: Option<SoftwareUpdateProviderConfig>,
    pub system_info: Option<SystemInfoConfig>,
    /// TLS is required if debug setting is false.
    pub tls: Option<TlsConfig>,
    /// Write timestamp to log messages. Enabled by default.
    pub log_timestamp: Option<bool>,
}

impl ConfigFile {
    pub fn save_default(dir: impl AsRef<Path>) -> Result<(), ConfigFileError> {
        let file_path =
            Self::default_config_file_path(dir).change_context(ConfigFileError::SaveDefault)?;
        let mut file =
            std::fs::File::create(file_path).change_context(ConfigFileError::SaveDefault)?;
        file.write_all(DEFAULT_CONFIG_FILE_TEXT.as_bytes())
            .change_context(ConfigFileError::SaveDefault)?;
        Ok(())
    }

    pub fn save_default_if_not_exist_and_load(
        dir: impl AsRef<Path>,
    ) -> Result<ConfigFile, ConfigFileError> {
        Self::load(dir, true)
    }

    pub fn load_config(dir: impl AsRef<Path>) -> Result<ConfigFile, ConfigFileError> {
        Self::load(dir, false)
    }

    fn load(dir: impl AsRef<Path>, save_default: bool) -> Result<ConfigFile, ConfigFileError> {
        let file_path =
            Self::default_config_file_path(&dir).change_context(ConfigFileError::LoadConfig)?;
        if !file_path.exists() && save_default {
            Self::save_default(dir).change_context(ConfigFileError::LoadConfig)?;
        }

        let config_string =
            std::fs::read_to_string(file_path).change_context(ConfigFileError::LoadConfig)?;
        toml::from_str(&config_string).change_context(ConfigFileError::LoadConfig)
    }

    pub fn default_config_file_path(dir: impl AsRef<Path>) -> Result<PathBuf, ConfigFileError> {
        if !dir.as_ref().is_dir() {
            return Err(Report::new(ConfigFileError::NotDirectory));
        }
        let mut file_path = dir.as_ref().to_path_buf();
        file_path.push(CONFIG_FILE_NAME);
        Ok(file_path)
    }

    pub fn exists(dir: impl AsRef<Path>) -> Result<bool, ConfigFileError> {
        let file_path =
            Self::default_config_file_path(&dir).change_context(ConfigFileError::LoadConfig)?;
        Ok(file_path.exists())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SocketConfig {
    pub public_api: SocketAddr,
    pub second_public_api_localhost_only_port: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TlsConfig {
    pub public_api_cert: PathBuf,
    pub public_api_key: PathBuf,

    /// Root certificate for HTTP client for checking API calls.
    pub root_certificate: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerEncryptionKey {
    pub manager_name: ManagerInstanceName,
    pub key_path: PathBuf,
}

impl ServerEncryptionKey {
    pub async fn read_encryption_key(&self) -> Result<SecureStorageEncryptionKey, GetConfigError> {
        tokio::fs::read_to_string(self.key_path.as_path())
            .await
            .change_context(GetConfigError::EncryptionKeyLoadingFailed)
            .map(|key| SecureStorageEncryptionKey {
                key: key.trim().to_string(),
            })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SecureStorageConfig {
    /// Name of manager instance which stores the encryption key
    /// for the secure storage.
    pub key_storage_manager_name: ManagerInstanceName,
    /// Path to file or directory which is used to
    /// check if the secure storage is mounted or not.
    pub availability_check_path: PathBuf,
    /// Optional. If the manager instance is not available, this key
    /// will be used for opening the encryption.
    /// Should not be used in production.
    pub encryption_key_text: Option<String>,
    /// Optional. Configure timeout for downloading the encryption key.
    pub key_download_timeout_seconds: Option<u32>,
}

// TODO: Make build and update configs generic. Does API need changes?

#[derive(Debug, Deserialize, Serialize)]
pub struct SoftwareUpdateProviderConfig {
    /// Manager instance URL which is used to
    /// check if new software is available.
    pub manager_base_url: Url,
    pub backend_install_location: PathBuf,
    /// Optional. Enableds data reset support for backend. This
    /// directory will be moved next to the original dir with postfix
    /// "-old" when backend is updated. If there is already a directory
    /// with that name, it will be deleted.
    pub backend_data_reset_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RebootIfNeededConfig {
    /// Time when reboot should be done. Format "hh:mm". For example "12:00".
    ///
    /// This is an UTC time value without UTC offset.
    pub time: UtcTimeValue,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SystemInfoConfig {
    pub log_services: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ManagerInstance {
    pub manager_name: ManagerInstanceName,
    pub url: Url,
}
