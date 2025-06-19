use std::{
    io::Write,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use error_stack::{Report, Result, ResultExt};
use manager_model::{ManagerInstanceName, SecureStorageEncryptionKey};
use serde::{Deserialize, Serialize};
use simple_backend_utils::{
    ContextExt,
    file::FileSizeValue,
    time::{DurationValue, UtcTimeValue},
};
use url::Url;

use super::GetConfigError;

pub const CONFIG_FILE_NAME: &str = "manager_config.toml";

pub const DEFAULT_CONFIG_FILE_TEXT: &str = r#"

# [manager]
# name = "default"
# api_key = "password"

# [dir]
# storage = "/afrodite-secure-storage/afrodite/manager"
# scripts = "/afrodite-server-tools/manager-tools"

# [general]
# log_timestamp = true # Optional

# [control_backend]
# service = "afrodite-backend.service"

[socket]
public_api = "127.0.0.1:4000"
# Second API has no TLS even if it is configured
# second_public_api_localhost_only_port = 4001

# [[remote_manager]]
# name = "backup"
# url = "tls://127.0.0.1:4000"

# [json_rpc_link.client]
# password = "password"
# url = "tls://127.0.0.1:4000"

# [json_rpc_link.server]
# name = "backup"
# password = "password"

# [backup_link.target]
# password = "password"
# url = "tls://127.0.0.1:4000"
# file_backup_retention_time = "30d" # Default

# [backup_link.server]
# password_target = "password"
# password_source = "password"

# [secure_storage]
# key_storage_manager_name = "default"
# availability_check_path = "/afrodite-secure-storage/afrodite"
# dir = "/afrodite-secure-storage"
# -------- Optional --------
# Fall back to local encryption key if the manager instance is not available.
# Should not be used in production.
# encryption_key_text = ""
# extend_size_to = "1G"

# [[server_encryption_key]]
# manager_name = "default"
# key_path = "data-key.key"

# [software_update]
# backend_install_location = "/home/afrodite/afrodite-backend"

# [software_update.github]
# owner = "TODO"
# repository = "TODO"
# file_name_ending = "TODO"
# uploader = "TODO" # Optional
# archive_backend_binary_path = "TODO" # Optional

# [manual_tasks]
# allow_backend_restart = true
# allow_system_reboot = true

# [manual_tasks.allow_backend_data_reset]
# backend_data_dir = "/path/to/backend/data"

# [scheduled_tasks]
# daily_start_time = "12:00"
# allow_backend_restart = true
# allow_system_reboot = true

# [automatic_system_reboot]
# scheduling_time = "11:00"
# notify_backend = true

# [system_info]
# log_services = ["afrodite-manager", "afrodite-backend"]

# [tls]
# public_api_cert = "manager-tls/server.crt"
# public_api_key = "manager-tls/server.key"
# root_cert = "manager-tls/root.crt"
"#;

#[derive(thiserror::Error, Debug)]
pub enum ConfigFileError {
    #[error("Save default")]
    SaveDefault,
    #[error("Not a directory")]
    NotDirectory,
    #[error("Load config file")]
    LoadConfig,
    #[error("Invalid config")]
    InvalidConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pub manager: ManagerConfig,
    pub dir: DirConfig,

    // Optional configs
    #[serde(default)]
    pub socket: SocketConfig,
    #[serde(default)]
    pub general: GeneralConfig,
    pub control_backend: Option<ControlBackendConfig>,
    #[serde(default)]
    pub remote_manager: Vec<ManagerInstance>,
    #[serde(default)]
    pub json_rpc_link: JsonRpcLinkConfig,
    #[serde(default)]
    pub backup_link: BackupLinkConfig,
    #[serde(default)]
    pub server_encryption_key: Vec<ServerEncryptionKey>,
    pub secure_storage: Option<SecureStorageConfig>,
    pub manual_tasks: Option<ManualTasksConfig>,
    pub scheduled_tasks: Option<ScheduledTasksConfig>,
    pub automatic_system_reboot: Option<AutomaticSystemRebootConfig>,
    pub software_update: Option<SoftwareUpdateConfig>,
    pub system_info: Option<SystemInfoConfig>,
    /// TLS is required if debug setting is false.
    pub tls: Option<TlsConfig>,
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
        let file: ConfigFile =
            toml::from_str(&config_string).change_context(ConfigFileError::LoadConfig)?;

        let system_reboot_scheduled_tasks_enabled = file
            .scheduled_tasks
            .as_ref()
            .map(|v| v.allow_system_reboot)
            .unwrap_or_default();
        if file.automatic_system_reboot.is_some() && !system_reboot_scheduled_tasks_enabled {
            return Err(ConfigFileError::InvalidConfig.report()).attach_printable(
                "Automatic system reboot requires enabling scheduled tasks with system reboot",
            );
        }

        Ok(file)
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
pub struct ManagerConfig {
    /// API key for manager API. All managers instances must use the same key.
    ///
    /// If the key is wrong the API access is denied untill manager is restarted.
    pub api_key: String,
    pub name: ManagerInstanceName,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DirConfig {
    /// Directory for build and update files.
    pub storage: PathBuf,
    pub scripts: PathBuf,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct GeneralConfig {
    /// Write timestamp to log messages. Enabled by default.
    log_timestamp: Option<bool>,
    debug: Option<bool>,
}

impl GeneralConfig {
    pub fn log_timestamp(&self) -> bool {
        self.log_timestamp.unwrap_or(true)
    }

    pub fn debug(&self) -> bool {
        self.debug.unwrap_or_default()
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SocketConfig {
    /// TCP or TLS depending on is TLS configured
    pub public_api: Option<SocketAddr>,
    /// TCP only
    pub second_public_api_localhost_only_port: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TlsConfig {
    pub public_api_cert: PathBuf,
    pub public_api_key: PathBuf,

    /// Root certificate for HTTP client for checking API calls.
    pub root_cert: PathBuf,
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
    /// Secure storage root directory. The secure-storage.sh script
    /// defines possible values for this.
    pub dir: PathBuf,
    /// Optional. If the manager instance is not available, this key
    /// will be used for opening the encryption.
    /// Should not be used in production.
    pub encryption_key_text: Option<String>,
    /// Optional. Configure timeout for downloading the encryption key.
    pub key_download_timeout_seconds: Option<u32>,
    /// Optional. Extend secure storage size.
    pub extend_size_to: Option<FileSizeValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SoftwareUpdateConfig {
    pub backend_install_location: PathBuf,
    pub github: SoftwareUpdateGitHubConfig,
}

/// Download latest release from GitHub repository.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SoftwareUpdateGitHubConfig {
    /// GitHub API access token for private repositories.
    pub token: Option<String>,
    pub owner: String,
    pub repository: String,
    /// Select release asset which name ends with this text.
    pub file_name_ending: String,
    /// Require specific release asset uploader.
    pub uploader: Option<String>,
    /// Selected release asset is an `.tar.gz`
    /// archive and backend binary is in the archive at
    /// this path.
    ///
    /// If path component is `*` then any string will match it.
    /// If the archive has multiple matching files the extracting
    /// will fail.
    pub archive_backend_binary_path: Option<SimplePatternPath>,
}

/// File path where path components might be equal with `*` string value.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SimplePatternPath(pub PathBuf);

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ManualTasksConfig {
    /// Allow manual backend data reset
    pub allow_backend_data_reset: Option<BackendDataResetConfig>,
    /// Allow manual backend restart
    pub allow_backend_restart: bool,
    /// Allow manaual system reboot
    pub allow_system_reboot: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackendDataResetConfig {
    /// This directory will be moved next to the original dir with postfix
    /// "-old" when backend is updated. If there is already a directory
    /// with that name, it will be deleted.
    pub backend_data_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScheduledTasksConfig {
    /// Time when scheduled tasks are started.
    ///
    /// This is an UTC time value without UTC offset and with format
    /// "hh:mm". For example "12:00".
    pub daily_start_time: UtcTimeValue,
    /// Allow scheduled backend restart
    pub allow_backend_restart: bool,
    /// Allow scheduled backend restart
    pub allow_system_reboot: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AutomaticSystemRebootConfig {
    pub scheduling_time: UtcTimeValue,
    pub notify_backend: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SystemInfoConfig {
    pub log_services: Vec<String>,
}

/// Manager instance which has server enabled
#[derive(Debug, Deserialize, Serialize)]
pub struct ManagerInstance {
    pub name: ManagerInstanceName,
    pub url: Url,
}

/// Create connection between managers for
/// JSON RPCs from server to client.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct JsonRpcLinkConfig {
    pub client: Option<JsonRpcLinkConfigClient>,
    pub server: Option<JsonRpcLinkConfigServer>,
}

impl JsonRpcLinkConfig {
    pub fn accepted_login_name(&self) -> Option<&ManagerInstanceName> {
        self.server.as_ref().map(|v| &v.name)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcLinkConfigServer {
    /// Accepted login name
    pub name: ManagerInstanceName,
    /// Accepted login password
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcLinkConfigClient {
    /// Link login password
    pub password: String,
    /// Manager server address
    pub url: Url,
}

/// Create connection between remote manager and backend for
/// transferring backups.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BackupLinkConfig {
    pub target: Option<BackupLinkConfigTarget>,
    pub server: Option<BackupLinkConfigServer>,
}

impl BackupLinkConfig {
    pub fn accepted_password_target(&self) -> Option<&str> {
        self.server.as_ref().map(|v| v.password_target.as_str())
    }

    pub fn accepted_password_source(&self) -> Option<&str> {
        self.server.as_ref().map(|v| v.password_source.as_str())
    }

    pub fn file_backup_retention_time(&self) -> DurationValue {
        self.target
            .as_ref()
            .and_then(|v| v.file_backup_retention_time)
            .unwrap_or(DurationValue::from_days(30))
    }
}

/// Remote manager which connects to server and receives the backups.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackupLinkConfigTarget {
    /// Link login password
    pub password: String,
    /// Manager server address
    pub url: Url,
    file_backup_retention_time: Option<DurationValue>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BackupLinkConfigServer {
    /// Accepted password for client which receives the backups
    pub password_target: String,
    /// Accepted password for client which sends the backups
    pub password_source: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ControlBackendConfig {
    /// Backend service name which `systemctl-access.sh` script accepts.
    pub service: String,
}
