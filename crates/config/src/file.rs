use std::{
    io::Write,
    net::SocketAddr,
    num::NonZeroU8,
    path::{Path, PathBuf},
};

use error_stack::{Report, Result, ResultExt};
use serde::{Deserialize, Serialize};
use url::Url;

pub type GoogleAccountId = String;

// Kilpisjärvi ja Nuorgam
// latitude_top_left = 70.1
// longitude_top_left = 20.5
//
// Eckerö (Ahvenanmaa) ja Nuorgam
// latitude_top_left = 70.1
// longitude_top_left = 19.5

// Suomen eteläisin kärki (Hanko) ja Suomen itäisin piste
// latitude_bottom_right = 59.8
// longitude_bottom_right = 31.58

pub const CONFIG_FILE_NAME: &str = "server_config.toml";

// Optional configs not in default file for safety:
// debug = false
//

pub const DEFAULT_CONFIG_FILE_TEXT: &str = r#"

# Also google account ID is required if sign in with google is enabled.
admin_email = "admin@example.com"

[location]
latitude_top_left = 70.1
longitude_top_left = 19.5
latitude_bottom_right = 59.8
longitude_bottom_right = 31.58
index_cell_square_km = 1

[socket]
public_api = "127.0.0.1:3000"
internal_api = "127.0.0.1:3001"

[database]
dir = "database"

[components]
account = true
profile = true
media = true
chat = true

# [manager]
# address = "http://127.0.0.1:5000"
# api_key = "TODO"

# [tile_map]
# tile_dir = "/map_tiles"

# [internal_api]
# Enable login and register route for bots
# bot_login = false

# [external_services]
# account_internal = "http://127.0.0.1:4000"
# media_internal = "http://127.0.0.1:4000"

# [sign_in_with_google]
# client_id_android = "id"
# client_id_ios = "id"
# client_id_server = "id"
# admin_google_account_id = "TODO"

# [tls]
# public_api_cert = "server_config/public_api.cert"
# public_api_key = "server_config/public_api.key"
# internal_api_cert = "server_config/internal_api.cert"
# internal_api_key = "server_config/internal_api.key"
# root_certificate = "server_config/root_certificate"

# [queue_limits]
# image_upload = 10

# [media_backup]
# ssh_address = "user@192.168.64.1"
# target_location = "/home/user/media_backup"
# ssh_private_key = "/home/local/.ssh/id_ed25519"
# rsync_time = "7:00"

# Backup SQLite database files using litestream tool
# [litestream]
# binary = "/usr/bin/litestream"
# config_file = "litestream.yml"

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
    #[error("Editing config file failed")]
    EditConfig,
    #[error("Saving edited config file failed")]
    SaveEditedConfig,
}

impl ConfigFileError {
    #[track_caller]
    pub fn report(self) -> error_stack::Report<Self> {
        error_stack::report!(self)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pub debug: Option<bool>,
    pub admin_email: String,
    pub components: Components,
    pub database: DatabaseConfig,
    pub socket: SocketConfig,
    pub location: LocationConfig,
    pub tile_map: Option<TileMapConfig>,
    pub manager: Option<AppManagerConfig>,
    pub external_services: Option<ExternalServices>,
    pub sign_in_with_google: Option<SignInWithGoogleConfig>,
    /// TLS is required if debug setting is false.
    pub tls: Option<TlsConfig>,

    pub internal_api: Option<InternalApiConfig>,
    pub queue_limits: Option<QueueLimitsConfig>,
    pub media_backup: Option<MediaBackupConfig>,
    pub litestream: Option<LitestreamConfig>,
}

impl ConfigFile {
    pub fn load(dir: impl AsRef<Path>) -> Result<ConfigFile, ConfigFileError> {
        let config_string = ConfigFileUtils::load_string(
            dir,
            CONFIG_FILE_NAME,
            DEFAULT_CONFIG_FILE_TEXT,
        )?;
        toml::from_str(&config_string).change_context(ConfigFileError::LoadConfig)
    }
}

pub struct ConfigFileUtils;

impl ConfigFileUtils {
    pub fn save_string(
        file_path: impl AsRef<Path>,
        text: &str,
    ) -> Result<(), ConfigFileError> {
        let mut file =
            std::fs::File::create(file_path).change_context(ConfigFileError::Save)?;
        file.write_all(text.as_bytes())
            .change_context(ConfigFileError::Save)?;
        Ok(())
    }

    pub fn join_dir_path_and_file_name(dir: impl AsRef<Path>, file_name: &str) -> Result<PathBuf, ConfigFileError> {
        if !dir.as_ref().is_dir() {
            return Err(Report::new(ConfigFileError::NotDirectory));
        }
        let mut file_path = dir.as_ref().to_path_buf();
        file_path.push(file_name);
        return Ok(file_path);
    }

    pub fn load_string(dir: impl AsRef<Path>, file_name: &str, default: &str) -> Result<String, ConfigFileError> {
        let file_path =
            Self::join_dir_path_and_file_name(&dir, file_name)
                .change_context(ConfigFileError::LoadConfig)?;
        if !file_path.exists() {
            Self::save_string(&file_path, default)
                .change_context(ConfigFileError::SaveDefault)?;
        }

        std::fs::read_to_string(&file_path)
            .change_context(ConfigFileError::LoadConfig)
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Components {
    pub account: bool,
    pub profile: bool,
    pub media: bool,
    pub chat: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub dir: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SocketConfig {
    pub public_api: SocketAddr,
    pub internal_api: SocketAddr,
}

/// App manager config
#[derive(Debug, Deserialize, Serialize)]
pub struct AppManagerConfig {
    pub address: Url,
    pub api_key: String,
}

/// Base URLs for external services
#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct ExternalServices {
    pub account_internal: Option<Url>,
    pub media_internal: Option<Url>,
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct TileMapConfig {
    /// Directory for map tiles.
    /// Tiles must be stored in z/x/y.png format.
    pub tile_dir: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LocationConfig {
    /// "y-axis" angle for top left corner of the location index.
    pub latitude_top_left: f64,
    /// "x-axis" angle for top left corner of the location index.
    pub longitude_top_left: f64,
    /// Minimun "y-axis" angle for bottom right corner of the location index.
    /// Index can in reality end further away.
    pub latitude_bottom_right: f64,
    /// Minimum "x-axis" angle for top left corner of the location index.
    /// Index can in reality end further away.
    pub longitude_bottom_right: f64,
    /// Index cell map size.
    pub index_cell_square_km: NonZeroU8,
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct InternalApiConfig {
    /// Enable register and login HTTP routes for bots through internal API socket.
    /// Note that debug option with this makes no authentication logins possible.
    pub bot_login: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SignInWithGoogleConfig {
    pub client_id_android: String,
    pub client_id_ios: String,
    pub client_id_server: String,
    pub admin_google_account_id: GoogleAccountId,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TlsConfig {
    pub public_api_cert: PathBuf,
    pub public_api_key: PathBuf,
    pub internal_api_cert: PathBuf,
    pub internal_api_key: PathBuf,
    pub root_certificate: PathBuf,
}

/// Backup media files to remote server using SSH
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaBackupConfig {
    /// For example "user@host"
    pub ssh_address: SshAddress,
    /// Target media backup location on remote server.
    pub target_location: PathBuf,
    pub ssh_private_key: AbsolutePathNoWhitespace,
    pub rsync_time: TimeValue,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(try_from = "String")]
pub struct SshAddress {
    pub username: String,
    pub address: String,
}

impl TryFrom<String> for SshAddress {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let values = value.trim().split(&['@']).collect::<Vec<&str>>();
        match values[..] {
            [username, address] => Ok(Self {
                username: username.to_string(),
                address: address.to_string(),
            }),
            _ => Err(format!("Unknown values: {:?}", values)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(try_from = "String")]
pub struct TimeValue {
    pub hours: u8,
    pub minutes: u8,
}

impl TryFrom<String> for TimeValue {
    type Error = String;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let iter = value.trim().split(':');
        let values: Vec<&str> = iter.collect();
        match values[..] {
            [hours, minutes] => {
                let hours: u8 = hours
                    .parse()
                    .map_err(|e: std::num::ParseIntError| e.to_string())?;
                let minutes: u8 = minutes
                    .parse()
                    .map_err(|e: std::num::ParseIntError| e.to_string())?;
                Ok(TimeValue { hours, minutes })
            }
            _ => Err(format!("Unknown values: {:?}", values)),
        }
    }
}

/// Config for Litestream SQLite backups
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LitestreamConfig {
    /// Path to Litestream binary.
    pub binary: PathBuf,
    /// Path to Litestream config file.
    pub config_file: PathBuf,
}

/// Server queue limits
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QueueLimitsConfig {
    pub image_upload: usize,
}

impl Default for QueueLimitsConfig {
    fn default() -> Self {
        Self {
            image_upload: 10,
        }
    }
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
