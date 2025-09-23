#![deny(unsafe_code)]
#![deny(unused_must_use)]
#![deny(unused_features)]
#![warn(unused_crate_dependencies)]

pub mod args;
pub mod file;
pub mod ip;

use std::{
    fs,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
    vec,
};

use args::ServerModeArgs;
use error_stack::{Result, ResultExt};
use file::{
    FirebaseCloudMessagingConfig, ImageProcessingConfig, MaxMindDbConfig, ScheduledTasksConfig,
    SignInWithAppleConfig, TileMapConfig, VideoCallingConfig,
};
use ip::IpList;
use reqwest::Url;
use rustls_pemfile::certs;
use tokio_rustls::rustls::ServerConfig;

use self::file::{ManagerConfig, SignInWithGoogleConfig, SimpleBackendConfigFile, SocketConfig};

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
    #[error("Directory creation failed")]
    DirCreationError,
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
    sign_in_with_urls: SignInWithUrls,
    sqlite_in_ram: bool,

    // TLS
    public_api_tls_config: Option<Arc<ServerConfig>>,

    // IP info
    ip_lists: Vec<IpList>,
}

impl SimpleBackendConfig {
    pub fn load_from_file_with_in_ram_database() -> Self {
        get_config(
            ServerModeArgs {
                sqlite_in_ram: true,
                data_dir: None,
            },
            String::new(),
            String::new(),
            true,
        )
        .unwrap()
    }

    /// Directory where SQLite databases and other files are stored.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn databases(&self) -> &DatabaseInfo {
        &DATABASES
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
    /// * Swagger UI is enabled on local bot API port.
    /// * Disabling HTTPS is possbile.
    /// * SQLite in RAM mode is allowed.
    /// * Atomic boolean `RUNNING_IN_DEBUG_MODE` is set to `true`.
    pub fn debug_mode(&self) -> bool {
        self.file.general.debug.unwrap_or(false)
    }

    pub fn sign_in_with_urls(&self) -> &SignInWithUrls {
        &self.sign_in_with_urls
    }

    pub fn sign_in_with_apple_config(&self) -> Option<&SignInWithAppleConfig> {
        self.file.sign_in_with_apple.as_ref()
    }

    pub fn sign_in_with_google_config(&self) -> Option<&SignInWithGoogleConfig> {
        self.file.sign_in_with_google.as_ref()
    }

    pub fn firebase_cloud_messaging_config(&self) -> Option<&FirebaseCloudMessagingConfig> {
        self.file.firebase_cloud_messaging.as_ref()
    }

    pub fn manager_config(&self) -> Option<&ManagerConfig> {
        self.file.manager.as_ref()
    }

    pub fn public_api_tls_config(&self) -> Option<&Arc<ServerConfig>> {
        self.public_api_tls_config.as_ref()
    }

    pub fn lets_encrypt_config(&self) -> Option<&file::LetsEncryptConfig> {
        self.file.lets_encrypt.as_ref()
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

    pub fn log_timestamp(&self) -> bool {
        self.file.general.log_timestamp.unwrap_or(true)
    }

    pub fn email_sending(&self) -> Option<&file::EmailSendingConfig> {
        self.file.email_sending.as_ref()
    }

    pub fn scheduled_tasks(&self) -> ScheduledTasksConfig {
        self.file.scheduled_tasks.clone().unwrap_or_default()
    }

    pub fn file_package(&self) -> Option<&file::StaticFilePackageHostingConfig> {
        self.file.static_file_package_hosting.as_ref()
    }

    pub fn image_processing(&self) -> ImageProcessingConfig {
        self.file.image_processing.clone().unwrap_or_default()
    }

    pub fn debug_face_detection_result(&self) -> Option<bool> {
        self.file.general.debug_face_detection_result
    }

    pub fn ip_lists(&self) -> &[IpList] {
        &self.ip_lists
    }

    pub fn maxmind_db_config(&self) -> Option<&MaxMindDbConfig> {
        self.file.ip_info.maxmind_db.as_ref()
    }

    pub fn video_calling(&self) -> &VideoCallingConfig {
        &self.file.video_calling
    }

    pub fn parsed_file(&self) -> &SimpleBackendConfigFile {
        &self.file
    }
}

/// Read config file from current directory.
pub fn get_config(
    args_config: args::ServerModeArgs,
    backend_code_version: String,
    backend_semver_version: String,
    save_default_config_if_not_found: bool,
) -> Result<SimpleBackendConfig, GetConfigError> {
    let current_dir = std::env::current_dir().change_context(GetConfigError::GetWorkingDir)?;
    let file_config =
        file::SimpleBackendConfigFile::load(current_dir, save_default_config_if_not_found)
            .change_context(GetConfigError::LoadFileError)?;

    let data_dir = if let Some(dir) = args_config.data_dir {
        dir
    } else {
        file_config.data.dir.clone()
    };

    if let Some(config) = file_config.firebase_cloud_messaging.as_ref() {
        if !config.service_account_key_path.exists() {
            return Err(GetConfigError::InvalidConfiguration).attach_printable(
                "Firebase Cloud Messaging service account key file does not exist",
            );
        }
    }

    let public_api_tls_config = match file_config.tls.clone().and_then(|v| v.public_api) {
        Some(tls_config) => Some(Arc::new(generate_server_config(
            tls_config.key.as_path(),
            tls_config.cert.as_path(),
        )?)),
        None => None,
    };

    if public_api_tls_config.is_some() && file_config.lets_encrypt.is_some() {
        return Err(GetConfigError::TlsConfigMissing).attach_printable(
            "Only either TLS certificate or Let's Encrypt should be configured for public API",
        );
    }

    if file_config.socket.public_api_enabled()
        && public_api_tls_config.is_none()
        && file_config.lets_encrypt.is_none()
        && !file_config.general.debug.unwrap_or_default()
    {
        return Err(GetConfigError::TlsConfigMissing).attach_printable(
            "TLS certificate or Let's Encrypt must be configured if some public API is enabled when debug mode is false",
        );
    }

    if let Some(lets_encrypt_config) = file_config.lets_encrypt.as_ref() {
        if !lets_encrypt_config.cache_dir.exists() {
            fs::create_dir_all(&lets_encrypt_config.cache_dir)
                .change_context(GetConfigError::DirCreationError)?
        } else if !lets_encrypt_config.cache_dir.is_dir() {
            return Err(GetConfigError::InvalidConfiguration).attach_printable(
                "Let's Encrypt cache directory config does not point to a directory",
            );
        }

        for d in &lets_encrypt_config.domains {
            if d.trim().is_empty() {
                return Err(GetConfigError::InvalidConfiguration)
                    .attach_printable("Let's Encrypt domain list contains empty domain");
            }
        }

        if lets_encrypt_config.email.trim().is_empty() {
            return Err(GetConfigError::InvalidConfiguration)
                .attach_printable("Let's Encrypt email is empty");
        }

        if lets_encrypt_config
            .cache_dir
            .to_string_lossy()
            .trim()
            .is_empty()
        {
            return Err(GetConfigError::InvalidConfiguration)
                .attach_printable("Let's Encrypt cache directory config is empty");
        }
    }

    let sqlite_in_ram = if args_config.sqlite_in_ram {
        if file_config.general.debug.unwrap_or_default() {
            true
        } else {
            return Err(GetConfigError::SqliteInRamNotAllowed)
                .attach_printable("SQLite in RAM mode is not allowed when debug mode is off");
        }
    } else {
        false
    };

    let mut ip_lists = vec![];
    for l in &file_config.ip_info.list {
        ip_lists.push(IpList::new(l)?);
    }

    if let Some(template) = file_config
        .video_calling
        .jitsi_meet
        .as_ref()
        .and_then(|v| v.custom_url.as_ref())
    {
        if !template.contains("{room}") {
            return Err(GetConfigError::InvalidConfiguration)
                .attach_printable("{room} is missing from Jitsi Meet custom URL config");
        }
        if !template.contains("{jwt}") {
            return Err(GetConfigError::InvalidConfiguration)
                .attach_printable("{jwt} is missing from Jitsi Meet custom URL config");
        }
    }

    let config = SimpleBackendConfig {
        file: file_config,
        data_dir,
        sqlite_in_ram,
        sign_in_with_urls: SignInWithUrls::new()?,
        public_api_tls_config,
        backend_code_version,
        backend_semver_version,
        ip_lists,
    };

    if config.debug_mode() {
        RUNNING_IN_DEBUG_MODE
            .debug
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    Ok(config)
}

const APPLE_PUBLIC_KEY_URL: &str = "https://appleid.apple.com/auth/keys";
const GOOGLE_PUBLIC_KEY_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";

/// This exists to avoid URL parsing erros when backend is running.
#[derive(Debug, Clone)]
pub struct SignInWithUrls {
    /// Request to this should return JwkSet.
    pub apple_public_keys: Url,
    /// Request to this should return JwkSet.
    pub google_public_keys: Url,
}

impl SignInWithUrls {
    pub fn new() -> Result<Self, GetConfigError> {
        Ok(Self {
            apple_public_keys: Url::parse(APPLE_PUBLIC_KEY_URL)
                .change_context(GetConfigError::ConstUrlParsingFailed)?,
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

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .change_context(GetConfigError::CreateTlsConfig)?;

    configure_apln_protocols(&mut config);

    Ok(config)
}

pub fn configure_apln_protocols(config: &mut ServerConfig) {
    config.alpn_protocols.push(b"h2".to_vec());
    config.alpn_protocols.push(b"http/1.1".to_vec());
}

const DATABASES: DatabaseInfo = DatabaseInfo {
    current: SqliteDatabase { name: "current" },
    history: SqliteDatabase { name: "history" },
};

#[derive(Debug, Clone, Copy)]
pub struct DatabaseInfo {
    pub current: SqliteDatabase,
    pub history: SqliteDatabase,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SqliteDatabase {
    pub name: &'static str,
}
