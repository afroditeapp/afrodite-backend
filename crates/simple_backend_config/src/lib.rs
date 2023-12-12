#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod args;
pub mod file;

use std::{
    io::BufReader,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
    vec,
};

use error_stack::{Result, ResultExt};
use file::{DatabaseInfo, TileMapConfig};
use reqwest::Url;
use rustls_pemfile::{certs, rsa_private_keys};
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

use self::file::{
    AppManagerConfig, LitestreamConfig, MediaBackupConfig, SignInWithGoogleConfig,
    SimpleBackendConfigFile, SocketConfig,
};

/// Config file debug mode status.
///
/// Parse the config file before reading this value.
pub static RUNNING_IN_DEBUG_MODE: GlobalDebugFlag = GlobalDebugFlag {
    debug: AtomicBool::new(false),
};

pub struct GlobalDebugFlag {
    debug: AtomicBool,
}

impl GlobalDebugFlag {
    pub fn value(&self) -> bool {
        self.debug.load(std::sync::atomic::Ordering::Relaxed)
    }
}

pub use self::file::ConfigFileError;

#[derive(thiserror::Error, Debug)]
pub enum GetConfigError {
    #[error("Get working directory error")]
    GetWorkingDir,
    #[error("File loading failed")]
    LoadFileError,
    #[error("Load config file")]
    LoadConfig,

    #[error("Parsing String constant to Url failed.")]
    ConstUrlParsingFailed,

    #[error("TLS config is required when debug mode is off")]
    TlsConfigMissing,
    #[error("TLS config creation error")]
    CreateTlsConfig,
    #[error("SQLite in RAM mode is not allowed when debug mode is off")]
    SqliteInRamNotAllowed,
    #[error("Invalid configuration")]
    InvalidConfiguration,
}

#[derive(Debug, Clone)]
pub struct SimpleBackendConfig {
    file: SimpleBackendConfigFile,

    /// Backend version with git commit ID and other info.
    backend_code_version: String,
    /// Semver version of the backend.
    backend_semver_version: String,

    // Server related configs
    data_dir: PathBuf,
    databases: Vec<DatabaseInfo>,
    sign_in_with_urls: SignInWithUrls,
    sqlite_in_ram: bool,

    // TLS
    public_api_tls_config: Option<Arc<ServerConfig>>,
    internal_api_tls_config: Option<Arc<ServerConfig>>,
    root_certificate: Option<reqwest::Certificate>,
}

impl SimpleBackendConfig {
    /// Directory where SQLite databases and other files are stored.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn databases(&self) -> &Vec<DatabaseInfo> {
        &self.databases
    }

    pub fn socket(&self) -> &SocketConfig {
        &self.file.socket
    }

    pub fn sqlite_in_ram(&self) -> bool {
        self.sqlite_in_ram
    }

    /// Server should run in debug mode.
    ///
    /// Debug mode changes:
    /// * Swagger UI is enabled.
    /// * Internal API is available also at same port as the public API.
    /// * Disabling HTTPS is possbile.
    /// * SQLite in RAM mode is allowed.
    /// * Atomic boolean `RUNNING_IN_DEBUG_MODE` is set to `true`.
    pub fn debug_mode(&self) -> bool {
        self.file.debug.unwrap_or(false)
    }

    pub fn sign_in_with_urls(&self) -> &SignInWithUrls {
        &self.sign_in_with_urls
    }

    pub fn sign_in_with_google_config(&self) -> Option<&SignInWithGoogleConfig> {
        self.file.sign_in_with_google.as_ref()
    }

    pub fn manager_config(&self) -> Option<&AppManagerConfig> {
        self.file.manager.as_ref()
    }

    pub fn public_api_tls_config(&self) -> Option<&Arc<ServerConfig>> {
        self.public_api_tls_config.as_ref()
    }

    pub fn internal_api_tls_config(&self) -> Option<&Arc<ServerConfig>> {
        self.internal_api_tls_config.as_ref()
    }

    pub fn root_certificate(&self) -> Option<&reqwest::Certificate> {
        self.root_certificate.as_ref()
    }

    pub fn media_backup(&self) -> Option<&MediaBackupConfig> {
        self.file.media_backup.as_ref()
    }

    pub fn litestream(&self) -> Option<&LitestreamConfig> {
        self.file.litestream.as_ref()
    }

    pub fn backend_code_version(&self) -> &str {
        &self.backend_code_version
    }

    pub fn backend_semver_version(&self) -> &str {
        &self.backend_semver_version
    }

    pub fn tile_map(&self) -> Option<&TileMapConfig> {
        self.file.tile_map.as_ref()
    }
}

/// Read config file from current directory.
pub fn get_config(
    args_config: args::ServerModeArgs,
    backend_code_version: String,
    backend_semver_version: String,
) -> Result<SimpleBackendConfig, GetConfigError> {
    let current_dir = std::env::current_dir().change_context(GetConfigError::GetWorkingDir)?;
    let file_config = file::SimpleBackendConfigFile::load(&current_dir)
        .change_context(GetConfigError::LoadFileError)?;

    let data_dir = if let Some(dir) = args_config.data_dir {
        dir
    } else {
        file_config.data.dir.clone()
    };

    let public_api_tls_config = match file_config.tls.clone() {
        Some(tls_config) => Some(Arc::new(generate_server_config(
            tls_config.public_api_key.as_path(),
            tls_config.public_api_cert.as_path(),
        )?)),
        None => None,
    };

    let internal_api_tls_config = match file_config.tls.clone() {
        Some(tls_config) => Some(Arc::new(generate_server_config(
            tls_config.internal_api_key.as_path(),
            tls_config.internal_api_cert.as_path(),
        )?)),
        None => None,
    };

    if public_api_tls_config.is_none() && !file_config.debug.unwrap_or_default() {
        return Err(GetConfigError::TlsConfigMissing)
            .attach_printable("TLS must be configured when debug mode is false");
    }

    let root_certificate = match file_config.tls.clone() {
        Some(tls_config) => Some(load_root_certificate(&tls_config.root_certificate)?),
        None => None,
    };

    let sqlite_in_ram = if args_config.sqlite_in_ram {
        if file_config.debug.unwrap_or_default() {
            true
        } else {
            return Err(GetConfigError::SqliteInRamNotAllowed)
                .attach_printable("SQLite in RAM mode is not allowed when debug mode is off");
        }
    } else {
        false
    };

    let databases = file_config.data.get_databases()?;

    let config = SimpleBackendConfig {
        file: file_config,
        data_dir,
        databases,
        sqlite_in_ram,
        sign_in_with_urls: SignInWithUrls::new()?,
        public_api_tls_config,
        internal_api_tls_config,
        root_certificate,
        backend_code_version,
        backend_semver_version,
    };

    if config.debug_mode() {
        RUNNING_IN_DEBUG_MODE
            .debug
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    Ok(config)
}

#[derive(Debug, Clone)]
pub struct InternalApiUrls {
    pub account_base_url: Option<Url>,
    pub media_base_url: Option<Url>,
}

impl InternalApiUrls {
    pub fn new(account_base_url: Option<Url>, media_base_url: Option<Url>) -> Self {
        Self {
            account_base_url,
            media_base_url,
        }
    }
}

const GOOGLE_PUBLIC_KEY_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";

#[derive(Debug, Clone)]
pub struct SignInWithUrls {
    /// Request to this should return JwkSet.
    pub google_public_keys: Url,
}

impl SignInWithUrls {
    pub fn new() -> Result<Self, GetConfigError> {
        Ok(Self {
            google_public_keys: Url::parse(GOOGLE_PUBLIC_KEY_URL)
                .change_context(GetConfigError::ConstUrlParsingFailed)?,
        })
    }
}

fn generate_server_config(
    key_path: &Path,
    cert_path: &Path,
) -> Result<ServerConfig, GetConfigError> {
    let mut key_reader = BufReader::new(
        std::fs::File::open(key_path).change_context(GetConfigError::CreateTlsConfig)?,
    );
    let all_keys =
        rsa_private_keys(&mut key_reader).change_context(GetConfigError::CreateTlsConfig)?;

    let key = if let [key] = &all_keys[..] {
        PrivateKey(key.clone())
    } else if all_keys.is_empty() {
        return Err(GetConfigError::CreateTlsConfig).attach_printable("No key found");
    } else {
        return Err(GetConfigError::CreateTlsConfig).attach_printable("Only one key supported");
    };

    let mut cert_reader = BufReader::new(
        std::fs::File::open(cert_path).change_context(GetConfigError::CreateTlsConfig)?,
    );
    let all_certs = certs(&mut cert_reader).change_context(GetConfigError::CreateTlsConfig)?;
    let cert = if let [cert] = &all_certs[..] {
        Certificate(cert.clone())
    } else if all_certs.is_empty() {
        return Err(GetConfigError::CreateTlsConfig).attach_printable("No cert found");
    } else {
        return Err(GetConfigError::CreateTlsConfig).attach_printable("Only one cert supported");
    };

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth() // TODO: configure at some point
        .with_single_cert(vec![cert], key)
        .change_context(GetConfigError::CreateTlsConfig)?;

    Ok(config)
}

fn load_root_certificate(cert_path: &Path) -> Result<reqwest::Certificate, GetConfigError> {
    let mut cert_reader = BufReader::new(
        std::fs::File::open(cert_path).change_context(GetConfigError::CreateTlsConfig)?,
    );
    let all_certs = certs(&mut cert_reader).change_context(GetConfigError::CreateTlsConfig)?;
    let cert = if let [cert] = &all_certs[..] {
        reqwest::Certificate::from_der(&cert.clone())
    } else if all_certs.is_empty() {
        return Err(GetConfigError::CreateTlsConfig).attach_printable("No cert found");
    } else {
        return Err(GetConfigError::CreateTlsConfig).attach_printable("Only one cert supported");
    }
    .change_context(GetConfigError::CreateTlsConfig)?;
    Ok(cert)
}
