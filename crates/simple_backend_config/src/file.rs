use std::{
    io::Write,
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    path::{Path, PathBuf},
    str::FromStr,
};

use chrono::{Datelike, Utc};
use error_stack::{Report, Result, ResultExt};
use manager_model::ManagerInstanceName;
use serde::{Deserialize, Deserializer, Serialize};
use simple_backend_utils::{
    ContextExt,
    time::{ByteCount, DurationValue},
};
use url::Url;

pub const CONFIG_FILE_NAME: &str = "simple_backend.toml";

pub const DEFAULT_CONFIG_FILE_TEXT: &str = r#"

# [general]
# log_timestamp = true

[socket]
public_api = "127.0.0.1:3000"
local_bot_api_port = 3001

# Use SQLite with default settings
[database.sqlite]

# TODO(future): Add connection error handling to Postgres support if needed.
#               Postgres support should not be used in production
#               before error handling is implemented.
# [database.postgres]
# current = "postgres://user:password@localhost/current_db"
# history = "postgres://user:password@localhost/history_db"

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

# [sign_in_with.apple]
# ios_bundle_id = "id"
# service_id = "id"
# android_package_id = "id"

# [sign_in_with.google]
# client_id_android = "id"
# client_id_ios = "id"
# client_id_web = "id"
# client_id_server = "id"

# [push_notifications.fcm]
# service_account_key_path = "server_config/service_account_key.json"

# [push_notifications.apns]
# key_path = "server_config/apns_key.p8"
# key_id = "TODO"
# team_id = "TODO"
# ios_bundle_id = "TODO"
# production_servers = false

# [push_notifications.web]
# vapid_private_key_path = "server_config/vapid_key.pem"

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

# Configuring Let's Encrypt will create socket public_api:443 if public API
# is not on port 443.
#
# [lets_encrypt]
# domains = ["example.com"]
# email = "test@example.com"
# production_servers = false

# [static_file_package_hosting]
# package = "frontend.tar.gz"
# read_from_dir = "" # optional, by default disabled
# disable_ip_allowlist = false # optional
# ip_allowlist = [] # optional

# [image_processing]
# jpeg_quality = 60 # optional

# If face detection is not configured all images are marked to include a face
# [image_processing.seetaface]
# model_file = "model.bin"
# detection_threshold = 2.8
# pyramid_scale_factor = 0.5

# [image_processing.nsfw_detection]
# model_file = "model.onnx"

# [image_processing.nsfw_detection.thresholds]
# hentai = 0.9
# porn = 0.9

# [[ip_info.lists]]
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
    #[error("Invalid config")]
    InvalidConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimpleBackendConfigFile {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub socket: SocketConfig,
    #[serde(default)]
    pub push_notifications: PushNotificationConfig,
    #[serde(default)]
    pub sign_in_with: SignInWithConfig,

    pub database: DatabaseConfig,

    pub tile_map: Option<TileMapConfig>,
    pub manager: Option<ManagerConfig>,
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
            socket: SocketConfig {
                public_api: None,
                local_bot_api_port: None,
                debug_local_bot_api_ip: None,
            },
            database: DatabaseConfig::sqlite(),
            push_notifications: PushNotificationConfig::default(),
            sign_in_with: SignInWithConfig::default(),
            email_sending: None,
            tile_map: None,
            manager: None,
            tls: None,
            lets_encrypt: None,
            static_file_package_hosting: None,
            image_processing: None,
            ip_info: IpInfoConfig::default(),
            video_calling: VideoCallingConfig::default(),
        }
    }

    pub fn load_from_dir(
        dir: impl AsRef<Path>,
        save_default_if_not_found: bool,
    ) -> Result<SimpleBackendConfigFile, ConfigFileError> {
        let file_path = ConfigFileUtils::join_dir_path_and_file_name(&dir, CONFIG_FILE_NAME)
            .change_context(ConfigFileError::LoadConfig)?;
        if !file_path.exists() && save_default_if_not_found {
            ConfigFileUtils::save_string(&file_path, DEFAULT_CONFIG_FILE_TEXT)
                .change_context(ConfigFileError::SaveDefault)?;
        }
        Self::load(&file_path)
    }

    pub fn load(file_path: impl AsRef<Path>) -> Result<SimpleBackendConfigFile, ConfigFileError> {
        let config_string =
            std::fs::read_to_string(&file_path).change_context(ConfigFileError::LoadConfig)?;
        let config: SimpleBackendConfigFile =
            toml::from_str(&config_string).change_context(ConfigFileError::LoadConfig)?;

        if let Some(nsfw_detection) = config
            .image_processing
            .as_ref()
            .and_then(|v| v.nsfw_detection.as_ref())
            && nsfw_detection.thresholds == NsfwDetectionThresholds::default()
        {
            return Err(ConfigFileError::InvalidConfig
                .report()
                .attach_printable("Config image_processing.nsfw_detection.thresholds is empty"));
        }

        if let Some(config) = &config.static_file_package_hosting {
            if config.package.is_some() && config.package_dir.is_some() {
                return Err(ConfigFileError::InvalidConfig.report().attach_printable(
                    "static_file_package_hosting: both package and package_dir are configured",
                ));
            }
            if config.package.is_none() && config.package_dir.is_none() {
                return Err(ConfigFileError::InvalidConfig.report().attach_printable(
                    "static_file_package_hosting: package or package_dir must be configured",
                ));
            }
        }

        if config.database.sqlite.is_some() && config.database.postgres.is_some() {
            return Err(ConfigFileError::InvalidConfig
                .report()
                .attach_printable("database: both sqlite and postgres cannot be enabled"));
        }

        if config.database.sqlite.is_none() && config.database.postgres.is_none() {
            return Err(ConfigFileError::InvalidConfig
                .report()
                .attach_printable("database: both sqlite and postgres cannot be disabled"));
        }

        Ok(config)
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
    pub debug_face_detection_result: Option<bool>,
    /// Write timestamp to log messages. Enabled by default.
    pub log_timestamp: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SocketConfig {
    /// If API obfuscation is enabled and remote bot login
    /// is enabled, unobfuscated API access is added for
    /// remote bot accounts.
    pub public_api: Option<SocketAddr>,
    /// Bot register, login, remote login and unobfuscated API access.
    pub local_bot_api_port: Option<u16>,
    /// Bot register, login, remote login and unobfuscated API access.
    ///
    /// Overrides the default localhost IP address.
    pub debug_local_bot_api_ip: Option<IpAddr>,
}

impl SocketConfig {
    pub fn public_api_enabled(&self) -> bool {
        self.public_api.is_some()
    }
}

/// App manager config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManagerConfig {
    pub name: ManagerInstanceName,
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

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct SignInWithConfig {
    pub apple: Option<SignInWithAppleConfig>,
    pub google: Option<SignInWithGoogleConfig>,
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

/// Firebase Cloud Messaging config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FcmConfig {
    /// Path to service account key JSON file.
    pub service_account_key_path: PathBuf,
    #[serde(default)]
    pub debug_logging: bool,
}

impl FcmConfig {
    pub(crate) const TOKEN_CACHE_FILE_NAME: &str = "firebase_token_cache.json";
}

/// Apple Push Notification service config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApnsConfig {
    /// Path to ".p8" file
    pub key_path: PathBuf,
    pub key_id: String,
    pub team_id: String,
    /// Used as notification's topic
    pub ios_bundle_id: String,
    pub production_servers: bool,
    #[serde(default)]
    pub debug_logging: bool,
}

/// Web push notification config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebPushConfig {
    /// Path to VAPID private key file (PEM format)
    pub vapid_private_key_path: PathBuf,
    #[serde(default)]
    pub debug_logging: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct PushNotificationConfig {
    pub fcm: Option<FcmConfig>,
    pub apns: Option<ApnsConfig>,
    pub web: Option<WebPushConfig>,
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
    #[serde(default)]
    pub debug_logging: bool,
    #[serde(default)]
    pub debug_example_com_is_normal_email: bool,
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
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PublicApiTlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
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
}

impl LetsEncryptConfig {
    pub(crate) const CACHE_DIR_NAME: &str = "lets_encrypt_cache";
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StaticFilePackageHostingConfig {
    /// Path to tar.gz package.
    pub package: Option<PathBuf>,
    /// Path to tar.gz package directory which contains tar.gz files
    /// which have version strings like `v0.0.0` in file names.
    /// If directory contains multiple files the latest version
    /// is selected as the primary version.
    pub package_dir: Option<PathBuf>,
    #[serde(flatten, default)]
    pub acccess: IpAddressAccessConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IpAddressAccessConfig {
    #[serde(default)]
    pub allow_all_ip_addresses: bool,
    /// Allow access from specific IP addresses.
    #[serde(default)]
    pub ip_allowlist: Vec<IpAddr>,
    /// Allow access from specific IP countries.
    ///
    /// All strings are converted to uppercase as it is assumed that
    /// MaxMind DB contains uppercase country codes.
    #[serde(default, deserialize_with = "ip_country_allowlist_from_vec_string")]
    pub ip_country_allowlist: Vec<String>,
}

pub fn ip_country_allowlist_from_vec_string<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    Vec::<String>::deserialize(d).map(|v| {
        v.iter()
            .map(|v| v.to_ascii_uppercase())
            .collect::<Vec<String>>()
    })
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
    input.chars().filter(|&c| !whitelist.contains(c)).collect()
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageProcessingConfig {
    /// Jpeg quality value. Value is clamped between 1-100.
    /// Mozjpeg library recommends 60-80 values
    #[serde(default = "default_jpeg_quality")]
    jpeg_quality: u8,
    pub seetaface: Option<SeetaFaceConfig>,
    pub nsfw_detection: Option<NsfwDetectionConfig>,
    /// Make sure to use higer value than the server process nice
    /// value as lower values require privileges.
    pub process_nice_value: Option<i8>,
}

fn default_jpeg_quality() -> u8 {
    60
}

impl Default for ImageProcessingConfig {
    fn default() -> Self {
        Self {
            jpeg_quality: default_jpeg_quality(),
            seetaface: None,
            nsfw_detection: None,
            process_nice_value: None,
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
    debug_log_results: Option<bool>,
}

impl SeetaFaceConfig {
    pub fn debug_log_results(&self) -> bool {
        self.debug_log_results.unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NsfwDetectionConfig {
    pub model_file: PathBuf,
    /// Thresholds when an image is classified as NSFW.
    ///
    /// If a probability value is equal or greater than the related
    /// threshold then the image is classified as NSFW.
    pub thresholds: NsfwDetectionThresholds,
    debug_log_results: Option<bool>,
}

impl NsfwDetectionConfig {
    pub fn debug_log_results(&self) -> bool {
        self.debug_log_results.unwrap_or_default()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct NsfwDetectionThresholds {
    pub drawings: Option<f32>,
    pub hentai: Option<f32>,
    pub neutral: Option<f32>,
    pub porn: Option<f32>,
    pub sexy: Option<f32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct IpInfoConfig {
    #[serde(default)]
    pub lists: Vec<IpListConfig>,
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
    /// Template download URL for MMDB database file with IP country info.
    ///
    /// # Placeholder strings
    /// - `{YYYY}` - year
    /// - `{MM}` - month
    download_url: MaxMindDbDownloadUrlTemplate,
    pub redownload_after_days: Option<u16>,
}

impl MaxMindDbConfig {
    pub fn new_download_url(&self) -> Url {
        self.download_url.url()
    }

    pub fn is_download_gz_compressed(&self) -> bool {
        self.download_url.0.ends_with(".gz")
    }
}

/// Valid template for creating MaxMind database download URL
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(try_from = "String")]
#[serde(into = "String")]
struct MaxMindDbDownloadUrlTemplate(String);

impl MaxMindDbDownloadUrlTemplate {
    fn url(&self) -> Url {
        Self::create_download_url(&self.0).unwrap()
    }

    fn create_download_url(template: &str) -> std::result::Result<Url, String> {
        let current_time = Utc::now();
        let url = template
            .replace("{YYYY}", &current_time.year().to_string())
            .replace("{MM}", &format!("{:0>2}", current_time.month()));
        let prevent_character = |c: char| {
            if url.contains(c) {
                Err(format!("Extra '{c}' character detected"))
            } else {
                Ok(())
            }
        };
        prevent_character('{')?;
        prevent_character('}')?;
        Url::from_str(&url).map_err(|e| e.to_string())
    }
}

impl TryFrom<String> for MaxMindDbDownloadUrlTemplate {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::create_download_url(&value)?;
        Ok(Self(value))
    }
}

impl From<MaxMindDbDownloadUrlTemplate> for String {
    fn from(value: MaxMindDbDownloadUrlTemplate) -> Self {
        value.0
    }
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    sqlite: Option<SqliteConfig>,
    pub postgres: Option<PostgresConfig>,
}

impl DatabaseConfig {
    pub fn sqlite() -> Self {
        Self {
            sqlite: Some(SqliteConfig::default()),
            postgres: None,
        }
    }

    pub fn is_sqlite(&self) -> bool {
        self.postgres.is_none()
    }

    pub fn sqlite_config(&self) -> SqliteConfig {
        self.sqlite.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SqliteConfig {
    #[serde(default)]
    pub vacuum: SqliteVacuumConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SqliteVacuumConfig {
    /// Minimum wait time since database file creation before running VACUUM
    pub min_wait_time: DurationValue,
    /// Maximum wait time since database file creation before forcing VACUUM
    pub max_wait_time: DurationValue,
    /// Maximum free space in database file (calculated using free DB pages)
    pub max_free_space: ByteCount,
}

impl Default for SqliteVacuumConfig {
    fn default() -> Self {
        SqliteVacuumConfig {
            min_wait_time: DurationValue::from_days(30),
            max_wait_time: DurationValue::from_days(365),
            max_free_space: ByteCount::from_megabytes(10),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostgresConfig {
    pub current: Url,
    pub history: Url,
}
