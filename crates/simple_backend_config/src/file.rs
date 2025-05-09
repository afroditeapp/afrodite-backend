use std::{
    io::Write,
    net::SocketAddr,
    num::NonZeroU32,
    path::{Path, PathBuf},
    str::FromStr,
};

use error_stack::{Report, Result, ResultExt};
use manager_model::ManagerInstanceName;
use serde::{Deserialize, Serialize};
use simple_backend_utils::time::{DurationValue, TimeValue, UtcTimeValue};
use url::Url;

pub const CONFIG_FILE_NAME: &str = "simple_backend_config.toml";

pub const DEFAULT_CONFIG_FILE_TEXT: &str = r#"

# [general]
# log_timestamp = true

[socket]
public_api = "127.0.0.1:3000"
public_bot_api = "127.0.0.1:3001"
local_bot_api_port = 3002

# [manager]
# manager_name = "default"
# address = "tls://localhost:4000"
# api_key = "TODO"
# backup_link_password = "password"

# [manager.tls]
# client_auth_cert = "/home/afrodite/manager-tls/server.crt"
# client_auth_cert_private_key = "/home/afrodite/manager-tls/server.key"
# root_cert = "/home/afrodite/manager-tls/root.crt"

# [tile_map]
# tile_dir = "/map_tiles"

# [sign_in_with_apple]
# ios_bundle_id = "id"
# service_id = "id"
# android_package_id = "id"

# [sign_in_with_google]
# client_id_android = "id"
# client_id_ios = "id"
# client_id_web = "id"
# client_id_server = "id"

# [firebase_cloud_messaging]
# service_account_key_path = "server_config/service_account_key.json"
# token_cache_path = "firebase_token_cache.json"

# [email_sending]
# smtp_server_address = "smtp.example.com"
# use_starttls_instead_of_smtps = false # optional
# username = "username"
# password = "password"
# email_from_header = "Example <no-reply@example.com>"
# send_limit_per_minute = 1, # optional, by default no limit
# send_limit_per_day = 10,   # optional, by default no limit

# [tls.public_api]
# cert = "server_config/public_api.crt"
# key = "server_config/public_api.key"

# [tls.internal_api]
# cert = "server_config/internal_api.crt"
# key = "server_config/internal_api.key"
# root_cert = "server_config/root_cert.crt"

# Configuring Let's Encrypt will create socket public_api:443 if public API
# is not on port 443.
#
# [lets_encrypt]
# domains = ["example.com"]
# email = "test@example.com"
# production_servers = false
# cache_dir = "lets_encrypt_cache"

# [scheduled_tasks]
# daily_start_time = "3:00"

# [static_file_package_hosting]
# package = "frontend.tar.gz"
# read_from_dir = "" # optional, by default disabled

# [image_processing]
# jpeg_quality = 60 # optional

# If face detection is not configured all images are marked to include a face
# [image_processing.seetaface]
# model_file = "model.bin"
# detection_threshold = 2.8
# pyramid_scale_factor = 0.5

# [[ip_info.list]]
# name = "test"
# file = "ip-list.txt"

# [ip_info.maxmind_db]
# download_url = "example.com"

# [video_calling.jitsi_meet]
# url = "https://jitsi.example.com"
# jwt_secret = "TODO"
# jwt_aud = "afrodite"
# jwt_iss = "afrodite"
# jwt_validity_time = "1h"
# room_prefix = "Afrodite_meeting_"

"#;

// TODO(prod): Consider changing manager config
// remote_manager.manager_name to remote_manager.name and
// simple backend config manager.manager_name to manager.name.

#[derive(thiserror::Error, Debug)]
pub enum ConfigFileError {
    #[error("Save config file failed")]
    Save,
    #[error("Save default")]
    SaveDefault,
    #[error("Not a directory")]
    NotDirectory,
    #[error("Load config file")]
    LoadConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimpleBackendConfigFile {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub socket: SocketConfig,
    #[serde(default)]
    pub data: DataConfig,

    pub tile_map: Option<TileMapConfig>,
    pub manager: Option<ManagerConfig>,
    pub sign_in_with_apple: Option<SignInWithAppleConfig>,
    pub sign_in_with_google: Option<SignInWithGoogleConfig>,
    pub firebase_cloud_messaging: Option<FirebaseCloudMessagingConfig>,
    pub email_sending: Option<EmailSendingConfig>,

    /// Manual TLS certificates.
    ///
    /// TLS certificate or Let's Encrypt must be configured for public API
    /// when debug mode is disabled.
    pub tls: Option<TlsConfig>,
    /// Let's Encrypt TLS certificates for public API.
    ///
    /// TLS certificate or Let's Encrypt must be configured for public API
    /// when debug mode is disabled.
    pub lets_encrypt: Option<LetsEncryptConfig>,

    pub scheduled_tasks: Option<ScheduledTasksConfig>,
    pub static_file_package_hosting: Option<StaticFilePackageHostingConfig>,
    pub image_processing: Option<ImageProcessingConfig>,

    #[serde(default)]
    pub ip_info: IpInfoConfig,
    #[serde(default)]
    pub video_calling: VideoCallingConfig,
}

impl SimpleBackendConfigFile {
    pub fn minimal_config_for_api_doc_json() -> Self {
        Self {
            general: GeneralConfig::default(),
            data: DataConfig {
                dir: PathBuf::new(),
            },
            socket: SocketConfig {
                public_api: None,
                public_bot_api: None,
                local_bot_api_port: None,
                experimental_internal_api: None,
            },
            email_sending: None,
            tile_map: None,
            manager: None,
            sign_in_with_apple: None,
            sign_in_with_google: None,
            firebase_cloud_messaging: None,
            tls: None,
            lets_encrypt: None,
            scheduled_tasks: None,
            static_file_package_hosting: None,
            image_processing: None,
            ip_info: IpInfoConfig::default(),
            video_calling: VideoCallingConfig::default(),
        }
    }

    pub fn load(
        dir: impl AsRef<Path>,
        save_default_if_not_found: bool,
    ) -> Result<SimpleBackendConfigFile, ConfigFileError> {
        let config_string =
            ConfigFileUtils::load_string(dir, CONFIG_FILE_NAME, DEFAULT_CONFIG_FILE_TEXT, save_default_if_not_found)?;
        toml::from_str(&config_string).change_context(ConfigFileError::LoadConfig)
    }
}

pub struct ConfigFileUtils;

impl ConfigFileUtils {
    pub fn save_string(file_path: impl AsRef<Path>, text: &str) -> Result<(), ConfigFileError> {
        let mut file = std::fs::File::create(file_path).change_context(ConfigFileError::Save)?;
        file.write_all(text.as_bytes())
            .change_context(ConfigFileError::Save)?;
        Ok(())
    }

    pub fn join_dir_path_and_file_name(
        dir: impl AsRef<Path>,
        file_name: &str,
    ) -> Result<PathBuf, ConfigFileError> {
        if !dir.as_ref().is_dir() {
            return Err(Report::new(ConfigFileError::NotDirectory));
        }
        let mut file_path = dir.as_ref().to_path_buf();
        file_path.push(file_name);
        Ok(file_path)
    }

    pub fn load_string(
        dir: impl AsRef<Path>,
        file_name: &str,
        default: &str,
        save_default_if_not_found: bool,
    ) -> Result<String, ConfigFileError> {
        let file_path = Self::join_dir_path_and_file_name(&dir, file_name)
            .change_context(ConfigFileError::LoadConfig)?;
        if !file_path.exists() && save_default_if_not_found {
            Self::save_string(&file_path, default).change_context(ConfigFileError::SaveDefault)?;
        }

        std::fs::read_to_string(&file_path).change_context(ConfigFileError::LoadConfig)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GeneralConfig {
    pub debug: Option<bool>,
    /// Override face detection result with this value
    pub debug_override_face_detection_result: Option<bool>,
    /// Write timestamp to log messages. Enabled by default.
    pub log_timestamp: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataConfig {
    /// Data directory for SQLite databases and other files.
    pub dir: PathBuf,
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            dir: "data".to_string().into()
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SocketConfig {
    pub public_api: Option<SocketAddr>,
    /// Bot remote login and unobfuscated API access.
    pub public_bot_api: Option<SocketAddr>,
    /// Bot register, login, remote login and unobfuscated API access.
    pub local_bot_api_port: Option<u16>,
    /// Socket address for server to server API which is only used
    /// when backend is running in microservice mode.
    /// The microservice mode is not currently working properly.
    pub experimental_internal_api: Option<SocketAddr>,
}

impl SocketConfig {
    pub fn public_api_enabled(&self) -> bool {
        self.public_api.is_some() ||
        self.public_bot_api.is_some() ||
        self.experimental_internal_api.is_some()
    }
}

/// App manager config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManagerConfig {
    pub manager_name: ManagerInstanceName,
    pub address: Url,
    pub api_key: String,
    pub tls: Option<ManagerTlsConfig>,
    pub backup_link_password: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManagerTlsConfig {
    /// TLS certificate which manager instance will check
    pub client_auth_cert: PathBuf,
    /// Private key of TLS certificate which manager instance will check
    pub client_auth_cert_private_key: PathBuf,
    /// Manager instance's root TLS certificate
    pub root_cert: PathBuf,
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct TileMapConfig {
    /// Directory for map tiles.
    /// Tiles must be stored in z/x/y.png format.
    pub tile_dir: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SignInWithAppleConfig {
    pub ios_bundle_id: String,
    /// Sign in with Apple web login service ID. This value is in JWT token
    /// aud field when login happens using Android or web app.
    pub service_id: String,
    /// Android app package ID. This value is used in HTTP redirect
    /// back to the app.
    pub android_package_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SignInWithGoogleConfig {
    pub client_id_android: String,
    pub client_id_ios: String,
    pub client_id_web: String,
    pub client_id_server: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FirebaseCloudMessagingConfig {
    /// Path to service account key JSON file.
    pub service_account_key_path: PathBuf,
    /// Path where cache Firebase token cache JSON file will be created.
    pub token_cache_path: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmailSendingConfig {
    /// The SMTP server must have port 465 open for sending emails using
    /// implicit TLS.
    pub smtp_server_address: String,
    /// Use STARTTLS to start TLS connection on port 587 instead of implicit
    /// TLS.
    #[serde(default)]
    pub use_starttls_instead_of_smtps: bool,
    pub username: String,
    pub password: String,
    /// Email `From` header, for example `Example <no-reply@example.com>`
    pub email_from_header: EmailFromHeader,
    pub send_limit_per_minute: Option<NonZeroU32>,
    pub send_limit_per_day: Option<NonZeroU32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct EmailFromHeader(pub lettre::message::Mailbox);

impl From<EmailFromHeader> for String {
    fn from(value: EmailFromHeader) -> Self {
        value.0.to_string()
    }
}

impl std::convert::TryFrom<String> for EmailFromHeader {
    type Error = lettre::address::AddressError;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let mailbox = lettre::message::Mailbox::from_str(&value)?;
        Ok(Self(mailbox))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TlsConfig {
    pub public_api: Option<PublicApiTlsConfig>,
    pub internal_api: Option<InternalApiTlsConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PublicApiTlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InternalApiTlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
    /// Root certificate for internal API client
    pub root_cert: PathBuf,
}

/// Let's Encrypt configuration for public API. If public API is not on
/// port 443, then another socket is created on public_api:443 for Let's
/// Encrypt ACME challenge.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LetsEncryptConfig {
    pub domains: Vec<String>,
    /// Email for receiving sertificate related notifications
    /// from Let's Encrypt.
    pub email: String,
    /// Use Let's Encrypt's production servers for certificate generation.
    pub production_servers: bool,
    /// Cache dir for Let's Encrypt certificates.
    ///
    /// The directory is created automatically if it does not exist.
    pub cache_dir: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaticFilePackageHostingConfig {
    /// Path to tar.gz package.
    pub package: PathBuf,
    /// Read files only from specific directory in the package.
    pub read_from_dir: Option<String>,
}

/// Absolute path with no whitespace.
/// Also contains only valid UTF-8 characters.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(try_from = "String")]
pub struct AbsolutePathNoWhitespace {
    pub path: PathBuf,
}

impl TryFrom<String> for AbsolutePathNoWhitespace {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let path = PathBuf::from(value.trim());
        validate_path(&path)?;
        Ok(Self { path })
    }
}

const PATH_CHARACTERS_WHITELIST: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_./";

fn whitelist_chars(input: &str, whitelist: &str) -> String {
    let invalid_chars = input.chars().filter(|&c| !whitelist.contains(c)).collect();
    invalid_chars
}

fn validate_path(input: &Path) -> std::result::Result<(), String> {
    if !input.is_absolute() {
        return Err(format!("Path is not absolute: {}", input.display()));
    }

    let unaccepted = whitelist_chars(
        input.as_os_str().to_string_lossy().as_ref(),
        PATH_CHARACTERS_WHITELIST,
    );
    if !unaccepted.is_empty() {
        return Err(format!(
            "Invalid characters {} in path: {}",
            unaccepted,
            input.display()
        ));
    }

    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScheduledTasksConfig {
    pub daily_start_time: UtcTimeValue,
}

impl Default for ScheduledTasksConfig {
    fn default() -> Self {
        const DEFAULT_SCHEDULED_TASKS_TIME: TimeValue = TimeValue::new(3, 0);

        Self {
            daily_start_time: UtcTimeValue(DEFAULT_SCHEDULED_TASKS_TIME),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageProcessingConfig {
    /// Jpeg quality value. Value is clamped between 1-100.
    /// Mozjpeg library recommends 60-80 values
    #[serde(default = "default_jpeg_quality")]
    jpeg_quality: u8,
    pub seetaface: Option<SeetaFaceConfig>,
}

fn default_jpeg_quality() -> u8 {
    60
}

impl Default for ImageProcessingConfig {
    fn default() -> Self {
        Self {
            jpeg_quality: default_jpeg_quality(),
            seetaface: None,
        }
    }
}

impl ImageProcessingConfig {
    pub fn jpeg_quality(&self) -> f32 {
        self.jpeg_quality as f32
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SeetaFaceConfig {
    pub model_file: String,
    pub detection_threshold: f64,
    pub pyramid_scale_factor: f32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct IpInfoConfig {
    #[serde(default)]
    pub list: Vec<IpListConfig>,
    pub maxmind_db: Option<MaxMindDbConfig>,
}

/// IP list file
///
/// # Example file
/// ```text
/// # Comment
///
/// 192.168.0.1
/// 192.168.0.2-192.168.0.20
/// 192.168.1.0/24
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IpListConfig {
    pub name: String,
    pub file: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MaxMindDbConfig {
    /// Download URL for latest GeoLite2 database file with IP country info.
    /// Note that the URL must point to the latest database and not to
    /// some specific version so that the DB is updated weekly.
    pub download_url: Url,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct VideoCallingConfig {
    pub jitsi_meet: Option<JitsiMeetConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JitsiMeetConfig {
    pub url: Url,
    pub jwt_secret: String,
    pub jwt_aud: String,
    pub jwt_iss: String,
    pub jwt_validity_time: DurationValue,
    pub room_prefix: String,
    /// Template URL which contains "{room}" and "{jwt}".
    /// Client opens this URL when configured and Jitsi Meet App
    /// is not installed.
    pub custom_url: Option<String>,
}
