#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

use std::{
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
    vec,
};

use error_stack::{Result, ResultExt};
use file::{
    AutomaticSystemRebootConfig, BackupLinkConfig, ControlBackendConfig, JsonRpcLinkConfig,
    ManagerInstance, ScheduledTasksConfig,
};
use manager_api::{RootCertStore, TlsConfig};
use manager_model::ManagerInstanceName;
use rustls_pemfile::certs;
use tokio_rustls::rustls::{
    ServerConfig,
    pki_types::{CertificateDer, pem::PemObject},
    server::WebPkiClientVerifier,
};
use tracing::{info, warn};

use self::file::{
    ConfigFile, ManualTasksConfig, SecureStorageConfig, ServerEncryptionKey, SocketConfig,
    SoftwareUpdateConfig, SystemInfoConfig,
};

pub mod args;
pub mod file;

#[derive(thiserror::Error, Debug)]
pub enum GetConfigError {
    #[error("Get working directory error")]
    GetWorkingDir,
    #[error("File loading failed")]
    LoadFileError,
    #[error("Load config file")]
    LoadConfig,
    #[error("Check config file existance error")]
    CheckConfigFileExistanceError,
    #[error("File path related error")]
    FilePathError,
    #[error("Change directory failed")]
    ChangeDirectoryFailed,

    #[error("TLS config is required when debug mode is off")]
    TlsConfigMissing,
    #[error("TLS config creation error")]
    CreateTlsConfig,

    // Server runtime errors
    #[error("Encryption key loading failed")]
    EncryptionKeyLoadingFailed,

    #[error("Missing script")]
    ScriptLocationError,

    #[error("Invalid constant")]
    InvalidConstant,
    #[error("Certificate file reading failed")]
    ReadCertificateError,
}

#[derive(Debug)]
pub struct Config {
    /// Backend version with git commit ID and other info.
    backend_code_version: String,
    /// Semver version of the backend.
    backend_semver_version: String,
    /// Backend binary Cargo package name.
    ///
    /// Used in `User-Agent` HTTP header for GitHub API.
    backend_pkg_name: String,

    file: ConfigFile,
    script_locations: ScriptLocations,

    // TLS
    public_api_tls_config: Option<Arc<ServerConfig>>,
    client_tls_config: Option<TlsConfig>,
}

impl Config {
    pub fn socket(&self) -> &SocketConfig {
        &self.file.socket
    }

    /// Server should run in debug mode.
    ///
    /// Debug mode changes:
    /// * Disabling HTTPS is possbile.
    /// * Checking available scripts is disabled.
    /// * Reboot command will not run.
    pub fn debug_mode(&self) -> bool {
        self.file.general.debug()
    }

    pub fn encryption_keys(&self) -> &[ServerEncryptionKey] {
        self.file.server_encryption_keys.as_slice()
    }

    pub fn secure_storage_config(&self) -> Option<&SecureStorageConfig> {
        self.file.secure_storage.as_ref()
    }

    pub fn software_update_provider(&self) -> Option<&SoftwareUpdateConfig> {
        self.file.software_update.as_ref()
    }

    pub fn api_key(&self) -> &str {
        &self.file.manager.api_key
    }

    pub fn public_api_tls_config(&self) -> Option<&Arc<ServerConfig>> {
        self.public_api_tls_config.as_ref()
    }

    pub fn client_tls_config(&self) -> Option<TlsConfig> {
        self.client_tls_config.clone()
    }

    pub fn script_locations(&self) -> &ScriptLocations {
        &self.script_locations
    }

    pub fn system_info(&self) -> Option<&SystemInfoConfig> {
        self.file.system_info.as_ref()
    }

    /// Directory for build and update files
    pub fn storage_dir(&self) -> &Path {
        &self.file.dir.storage
    }

    pub fn manual_tasks_config(&self) -> ManualTasksConfig {
        self.file.manual_tasks.as_ref().cloned().unwrap_or_default()
    }

    pub fn scheduled_tasks(&self) -> Option<&ScheduledTasksConfig> {
        self.file.scheduled_tasks.as_ref()
    }

    pub fn automatic_system_reboot(&self) -> Option<&AutomaticSystemRebootConfig> {
        self.file.automatic_system_reboot.as_ref()
    }

    pub fn control_backend(&self) -> Option<&ControlBackendConfig> {
        self.file.control_backend.as_ref()
    }

    pub fn log_timestamp(&self) -> bool {
        self.file.general.log_timestamp()
    }

    pub fn backend_code_version(&self) -> &str {
        &self.backend_code_version
    }

    pub fn backend_semver_version(&self) -> &str {
        &self.backend_semver_version
    }

    pub fn backend_pkg_name(&self) -> &str {
        &self.backend_pkg_name
    }

    pub fn remote_managers(&self) -> &[ManagerInstance] {
        &self.file.remote_managers
    }

    pub fn json_rpc_link(&self) -> &JsonRpcLinkConfig {
        &self.file.json_rpc_link
    }

    pub fn backup_link(&self) -> &BackupLinkConfig {
        &self.file.backup_link
    }

    pub fn find_remote_manager(&self, name: &ManagerInstanceName) -> Option<&ManagerInstance> {
        self.remote_managers().iter().find(|v| v.name == *name)
    }

    pub fn manager_name(&self) -> ManagerInstanceName {
        self.file.manager.name.clone()
    }

    pub fn update_manager_user_agent(&self) -> String {
        format!(
            "{}/{}",
            self.backend_pkg_name(),
            self.backend_semver_version()
        )
    }

    pub fn parsed_file(&self) -> &ConfigFile {
        &self.file
    }
}

pub fn get_config(
    manager_config_file: PathBuf,
    backend_code_version: String,
    backend_semver_version: String,
    backend_pkg_name: String,
) -> Result<Config, GetConfigError> {
    let file_config =
        file::ConfigFile::save_default_if_not_exist_and_load_file(&manager_config_file)
            .change_context(GetConfigError::LoadFileError)?;

    let mut config_dir = manager_config_file
        .canonicalize()
        .change_context(GetConfigError::FilePathError)?;
    config_dir.pop();
    std::env::set_current_dir(config_dir).change_context(GetConfigError::ChangeDirectoryFailed)?;

    let public_api_tls_config = match file_config.tls.clone() {
        Some(tls_config) => Some(Arc::new(generate_server_config(
            tls_config.root_cert.as_path(),
            tls_config.public_api_cert.as_path(),
            tls_config.public_api_key.as_path(),
        )?)),
        None => None,
    };

    let client_tls_config = match file_config.tls.clone() {
        Some(tls_config) => {
            let config = TlsConfig::new(
                tls_config.root_cert,
                tls_config.public_api_cert,
                tls_config.public_api_key,
            )
            .change_context(GetConfigError::ReadCertificateError)?;
            Some(config)
        }
        None => None,
    };

    if public_api_tls_config.is_none() && !file_config.general.debug() {
        return Err(GetConfigError::TlsConfigMissing)
            .attach_printable("TLS must be configured when debug mode is false");
    }

    let script_locations =
        check_script_locations(&file_config.dir.scripts, file_config.general.debug())?;

    Ok(Config {
        backend_code_version,
        backend_semver_version,
        backend_pkg_name,
        file: file_config,
        script_locations,
        public_api_tls_config,
        client_tls_config,
    })
}

fn check_script_locations(
    script_dir: &Path,
    is_debug: bool,
) -> Result<ScriptLocations, GetConfigError> {
    let print_logs = script_dir.join("print-logs.sh");
    let secure_storage = script_dir.join("secure-storage.sh");
    let systemctl_access = script_dir.join("systemctl-access.sh");

    let mut errors = vec![];

    if !print_logs.exists() {
        errors.push(format!("Script not found: {}", print_logs.display()));
    }
    if !secure_storage.exists() {
        errors.push(format!("Script not found: {}", print_logs.display()));
    }
    if !systemctl_access.exists() {
        errors.push(format!("Script not found: {}", print_logs.display()));
    }

    if errors.is_empty() || is_debug {
        if errors.is_empty() {
            info!("All scripts found");
        } else {
            warn!("Some scripts are missing.\n{}", errors.join("\n"));
        }
        Ok(ScriptLocations {
            print_logs,
            secure_storage,
            systemctl_access,
        })
    } else {
        Err(GetConfigError::ScriptLocationError).attach_printable(errors.join("\n"))
    }
}

fn generate_server_config(
    root_cert_path: &Path,
    cert_path: &Path,
    key_path: &Path,
) -> Result<ServerConfig, GetConfigError> {
    let client_auth_root_certificate = CertificateDer::from_pem_file(root_cert_path)
        .change_context(GetConfigError::CreateTlsConfig)?;
    let mut client_auth_root_store = RootCertStore::empty();
    client_auth_root_store
        .add(client_auth_root_certificate)
        .change_context(GetConfigError::CreateTlsConfig)?;
    let client_verifier = WebPkiClientVerifier::builder(client_auth_root_store.into())
        .build()
        .change_context(GetConfigError::CreateTlsConfig)?;

    let mut key_reader = BufReader::new(
        std::fs::File::open(key_path).change_context(GetConfigError::CreateTlsConfig)?,
    );
    let all_keys: Vec<_> = rustls_pemfile::private_key(&mut key_reader)
        .iter()
        .flatten()
        .map(|v| v.clone_key())
        .collect();
    let mut key_iter = all_keys.into_iter();

    let key = if let Some(key) = key_iter.next() {
        key
    } else {
        return Err(GetConfigError::CreateTlsConfig).attach_printable("No key found");
    };

    if key_iter.next().is_some() {
        return Err(GetConfigError::CreateTlsConfig).attach_printable("Only one key supported");
    }

    let mut cert_reader = BufReader::new(
        std::fs::File::open(cert_path).change_context(GetConfigError::CreateTlsConfig)?,
    );

    let all_certs: Vec<_> = certs(&mut cert_reader)
        .map(|r| r.map(|c| c.into_owned()))
        .collect();
    let mut cert_iter = all_certs.into_iter();
    let cert = if let Some(cert) = cert_iter.next() {
        cert.change_context(GetConfigError::CreateTlsConfig)?
    } else {
        return Err(GetConfigError::CreateTlsConfig).attach_printable("No cert found");
    };

    if cert_iter.next().is_some() {
        return Err(GetConfigError::CreateTlsConfig).attach_printable("Only one cert supported");
    }

    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(vec![cert], key)
        .change_context(GetConfigError::CreateTlsConfig)?;

    Ok(config)
}

#[derive(Debug)]
pub struct ScriptLocations {
    pub print_logs: PathBuf,
    pub secure_storage: PathBuf,
    pub systemctl_access: PathBuf,
}

impl ScriptLocations {
    pub fn print_logs(&self) -> &Path {
        &self.print_logs
    }

    pub fn secure_storage(&self) -> &Path {
        &self.secure_storage
    }

    pub fn systemctl_access(&self) -> &Path {
        &self.systemctl_access
    }
}
