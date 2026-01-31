use std::{
    num::NonZeroU8,
    path::{Path, PathBuf},
};

use error_stack::{Result, ResultExt};
use model::ClientVersion;
// Re-export for test-mode crate
pub use model_server_data::EmailAddress;
use model_server_state::DemoAccountId;
use serde::{Deserialize, Serialize};
use simple_backend_config::file::IpAddressAccessConfig;
use simple_backend_model::VersionNumber;
use simple_backend_utils::time::{ByteCount, DurationValue, TimeValue, UtcTimeValue};

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

pub const DEFAULT_CONFIG_FILE_TEXT: &str = r#"

# [grant_admin_access]
# email = "admin@example.com"

# [location]
# latitude_top_left = 70.1
# longitude_top_left = 19.5
# latitude_bottom_right = 59.8
# longitude_bottom_right = 31.58
# index_cell_square_km = 255       # 1-255 and area width and height must be larger than 255 km

# [limits.common]
# send_report_daily_max_count = 20
#
# [limits.common.processed_report_deletion_wait_duration]
# profile_name = "90d"
# profile_text = "90d"

# [limits.account]
# inactivity_logout_wait_duration = "365d"
# account_deletion_wait_duration = "90d"
# init_deletion_for_inactive_accounts_wait_duration = "730d"
# email_verification_token_validity_duration = "1d"
# email_verification_resend_min_wait_duration = "15m"
# email_change_min_wait_duration = "3d"
# email_change_resend_min_wait_duration = "15m"
# email_login_token_validity_duration = "15m"
# email_login_resend_min_wait_duration = "15m"

# [limits.chat]
# max_public_key_count = 20
# send_message_daily_max_count = 200
# new_message_email_with_push_notification_device_token = "7d"
# new_message_email_without_push_notification_device_token = "1d"
# new_like_email_with_push_notification_device_token = "7d"
# new_like_email_without_push_notification_device_token = "1d"
# backup_transfer_yearly_max_bytes = "100M"

# [limits.media]
# max_content_count = 20
# unused_content_wait_duration = "90d"
# get_profile_content_info_daily_max_count = 2000

# [limits.profile]
# profile_iterator_reset_daily_max_count = 200
# profile_iterator_next_page_daily_max_count = 1000
# get_profile_daily_max_count = 2000

# [[profile_name_allowlists]]
# csv_file = "names.csv"
# delimiter = ";"
# column_index = 0
# start_row_index = 1

# [remote_bot_login]
# password = "TODO"

# [automatic_profile_search]
# daily_start_time = "9:00"
# daily_end_time = "21:00"

# Uncomment to enable client version tracking
# [api.client_version_tracking]
# major_min = 0
# major_max = 10
# minor_min = 0
# minor_max = 100
# patch_min = 0
# patch_max = 1000

"#;

#[derive(thiserror::Error, Debug)]
pub enum ConfigFileError {
    #[error("Simple backend error")]
    SimpleBackendError,

    #[error("Load config file")]
    LoadConfig,
    #[error("Editing config file failed")]
    EditConfig,
    #[error("Saving edited config file failed")]
    SaveEditedConfig,

    #[error("Invalid config")]
    InvalidConfig,
    #[error("Change directory failed")]
    ChangeDirectoryFailed,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub automatic_profile_search: AutomaticProfileSearchConfig,
    #[serde(default)]
    pub remote_bot_login: RemoteBotLoginConfig,

    pub grant_admin_access: Option<GrantAdminAccessConfig>,
    pub location: Option<LocationConfig>,
    pub demo_accounts: Option<Vec<DemoAccountConfig>>,
    pub limits: Option<LimitsConfig>,
    pub profile_name_allowlists: Option<Vec<ProfiletNameAllowlistConfig>>,
}

impl ConfigFile {
    pub const CONFIG_FILE_NAME: &str = "server.toml";

    pub fn minimal_config_for_api_doc_json() -> Self {
        Self {
            general: Default::default(),
            api: ApiConfig::default(),
            automatic_profile_search: AutomaticProfileSearchConfig::default(),
            remote_bot_login: RemoteBotLoginConfig::default(),
            grant_admin_access: None,
            location: None,
            demo_accounts: None,
            limits: None,
            profile_name_allowlists: None,
        }
    }

    pub fn default_file_path() -> Result<PathBuf, ConfigFileError> {
        let current_dir = std::env::current_dir().change_context(ConfigFileError::LoadConfig)?;
        Ok(current_dir.join(Self::CONFIG_FILE_NAME))
    }

    pub fn load_from_default_location(save_if_needed: bool) -> Result<Self, ConfigFileError> {
        let path = Self::default_file_path()?;
        if !path.exists() && save_if_needed {
            std::fs::write(&path, DEFAULT_CONFIG_FILE_TEXT)
                .change_context(ConfigFileError::LoadConfig)?;
        }
        Self::load(path)
    }

    pub fn load(file: impl AsRef<Path>) -> Result<Self, ConfigFileError> {
        let config_string =
            std::fs::read_to_string(file).change_context(ConfigFileError::LoadConfig)?;
        let file: Self =
            toml::from_str(&config_string).change_context(ConfigFileError::LoadConfig)?;

        Ok(file)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GeneralConfig {
    /// Allow using backend data reset API.
    #[serde(default)]
    pub debug_allow_backend_data_reset: bool,
    #[serde(default)]
    pub debug_websocket_logging: bool,
    #[serde(default)]
    pub debug_disable_api_limits: bool,
    /// Make sure to use higer value than the server process nice
    /// value as lower values require privileges.
    pub bot_process_nice_value: Option<i8>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ApiConfig {
    pub obfuscation_salt: Option<String>,
    pub min_client_version: Option<MinClientVersion>,
    pub client_version_tracking: Option<ClientVersionTrackingConfig>,
}

/// Client version tracking is disabled when these values are not
/// configured.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientVersionTrackingConfig {
    pub major_min: u32,
    pub major_max: u32,
    pub minor_min: u32,
    pub minor_max: u32,
    pub patch_min: u32,
    pub patch_max: u32,
}

impl ClientVersionTrackingConfig {
    pub fn is_valid(&self, version: ClientVersion) -> bool {
        version.major >= self.major_min
            && version.major <= self.major_max
            && version.minor >= self.minor_min
            && version.minor <= self.minor_max
            && version.patch >= self.patch_min
            && version.patch <= self.patch_max
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GrantAdminAccessConfig {
    /// Grant admin access to every new account which matches with email.
    ///
    /// By default admin access is granted for once only.
    #[serde(default)]
    pub debug_for_every_matching_new_account: bool,
    /// Change matching to check only email domain.
    #[serde(default)]
    pub debug_match_only_email_domain: bool,
    pub email: EmailAddress,
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
    /// Index cell map size target value. Might be smaller or larger depending
    /// the supported tile sizes.
    pub index_cell_square_km: NonZeroU8,
}

impl Default for LocationConfig {
    fn default() -> Self {
        Self {
            // Just use Finland as default as that is tested to work.
            latitude_top_left: 70.1,
            longitude_top_left: 19.5,
            latitude_bottom_right: 59.8,
            longitude_bottom_right: 31.58,
            // Make matrix cells 255 square kilometers, so the matrix will not
            // consume that much of memory.
            index_cell_square_km: NonZeroU8::MAX,
        }
    }
}

/// Limits config
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct LimitsConfig {
    pub common: Option<CommonLimitsConfig>,
    pub account: Option<AccountLimitsConfig>,
    pub chat: Option<ChatLimitsConfig>,
    pub media: Option<MediaLimitsConfig>,
    pub profile: Option<ProfileLimitsConfig>,
}

/// Common limits config for all server components
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommonLimitsConfig {
    pub processed_report_deletion_wait_duration: ProcessedReportDeletionConfig,
    pub send_report_daily_max_count: u16,
}

impl Default for CommonLimitsConfig {
    fn default() -> Self {
        Self {
            processed_report_deletion_wait_duration: ProcessedReportDeletionConfig::default(),
            send_report_daily_max_count: 20,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProcessedReportDeletionConfig {
    pub profile_name: DurationValue,
    pub profile_text: DurationValue,
}

impl Default for ProcessedReportDeletionConfig {
    fn default() -> Self {
        Self {
            profile_name: DurationValue::from_days(90),
            profile_text: DurationValue::from_days(90),
        }
    }
}

/// Account related limits config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AccountLimitsConfig {
    pub inactivity_logout_wait_duration: DurationValue,
    pub account_deletion_wait_duration: DurationValue,
    pub init_deletion_for_inactive_accounts_wait_duration: DurationValue,
    pub email_verification_token_validity_duration: DurationValue,
    pub email_verification_resend_min_wait_duration: DurationValue,
    /// After this time pending email address is changed to be
    /// current email address. The email change email
    /// verification token is valid during this [DurationValue].
    pub email_change_min_wait_duration: DurationValue,
    pub email_change_resend_min_wait_duration: DurationValue,
    pub email_login_token_validity_duration: DurationValue,
    pub email_login_resend_min_wait_duration: DurationValue,
}

impl Default for AccountLimitsConfig {
    fn default() -> Self {
        Self {
            inactivity_logout_wait_duration: DurationValue::from_days(365),
            account_deletion_wait_duration: DurationValue::from_days(90),
            init_deletion_for_inactive_accounts_wait_duration: DurationValue::from_days(365 * 2), // About 2 years
            email_verification_token_validity_duration: DurationValue::from_days(1),
            email_verification_resend_min_wait_duration: DurationValue::from_seconds(15 * 60),
            email_change_min_wait_duration: DurationValue::from_days(3),
            email_change_resend_min_wait_duration: DurationValue::from_seconds(15 * 60),
            email_login_token_validity_duration: DurationValue::from_seconds(15 * 60),
            email_login_resend_min_wait_duration: DurationValue::from_seconds(15 * 60),
        }
    }
}

/// Chat releated limits config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatLimitsConfig {
    pub max_public_key_count: u16,
    pub send_message_daily_max_count: u16,
    pub new_message_email_with_push_notification_device_token: DurationValue,
    pub new_message_email_without_push_notification_device_token: DurationValue,
    pub new_like_email_with_push_notification_device_token: DurationValue,
    pub new_like_email_without_push_notification_device_token: DurationValue,
    pub backup_transfer_yearly_max_bytes: ByteCount,
}

impl Default for ChatLimitsConfig {
    fn default() -> Self {
        Self {
            max_public_key_count: 20,
            send_message_daily_max_count: 200,
            new_message_email_with_push_notification_device_token: DurationValue::from_days(7),
            new_message_email_without_push_notification_device_token: DurationValue::from_days(1),
            new_like_email_with_push_notification_device_token: DurationValue::from_days(7),
            new_like_email_without_push_notification_device_token: DurationValue::from_days(1),
            backup_transfer_yearly_max_bytes: ByteCount::from_megabytes(100),
        }
    }
}

/// Media related limits config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaLimitsConfig {
    pub max_content_count: u8,
    pub unused_content_wait_duration: DurationValue,
    pub get_profile_content_info_daily_max_count: u16,
}

impl Default for MediaLimitsConfig {
    fn default() -> Self {
        Self {
            max_content_count: 20,
            unused_content_wait_duration: DurationValue::from_days(90),
            get_profile_content_info_daily_max_count: 2000,
        }
    }
}

/// Profile related limits config
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProfileLimitsConfig {
    /// Used also for automatic profile search specific iterator
    pub profile_iterator_reset_daily_max_count: u16,
    /// Used also for automatic profile search specific iterator
    pub profile_iterator_next_page_daily_max_count: u16,
    pub get_profile_daily_max_count: u16,
}

impl Default for ProfileLimitsConfig {
    fn default() -> Self {
        Self {
            profile_iterator_reset_daily_max_count: 200,
            profile_iterator_next_page_daily_max_count: 1000,
            get_profile_daily_max_count: 2000,
        }
    }
}

/// Demo account configuration.
///
/// Adding one or more demo account configurations
/// will enable demo account HTTP routes.
///
/// WARNING: Demo account gives access to all/specific accounts.
#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct DemoAccountConfig {
    pub database_id: DemoAccountId,
    pub username: String,
    /// If this is quessed wrong, these demo account credentials will
    /// be locked until server restarts.
    pub password: String,
    /// If true then all accounts are accessible.
    /// Overrides `accessible_accounts`.
    #[serde(default)]
    pub access_all_accounts: bool,
    /// Maximum number of normal accounts this demo account can create.
    max_account_count: Option<u8>,
    /// AccountIds for accounts that are accessible with demo account.
    #[serde(default)]
    pub accessible_accounts: Vec<simple_backend_utils::UuidBase64Url>,
}

impl DemoAccountConfig {
    pub fn max_account_count(&self) -> u8 {
        self.max_account_count.unwrap_or(10)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProfiletNameAllowlistConfig {
    pub csv_file: PathBuf,
    pub delimiter: char,
    /// Column index starting from zero.
    pub column_index: usize,
    /// Index for first row where data reading starts. The index values
    /// starts from zero.
    pub start_row_index: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct MinClientVersion(pub VersionNumber);

impl MinClientVersion {
    pub fn received_version_is_accepted(&self, received: ClientVersion) -> bool {
        Into::<VersionNumber>::into(received) >= self.0
    }
}

/// Remote bot login config
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct RemoteBotLoginConfig {
    pub password: Option<String>,
    #[serde(flatten)]
    pub access: Option<IpAddressAccessConfig>,
}

impl RemoteBotLoginConfig {
    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    pub fn access(&self) -> Option<&IpAddressAccessConfig> {
        self.access.as_ref()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AutomaticProfileSearchConfig {
    pub daily_start_time: UtcTimeValue,
    pub daily_end_time: UtcTimeValue,
}

impl Default for AutomaticProfileSearchConfig {
    fn default() -> Self {
        const DEFAULT_START_TIME: TimeValue = TimeValue::new(9, 0);
        const DEFAULT_END_TIME: TimeValue = TimeValue::new(21, 0);

        Self {
            daily_start_time: UtcTimeValue(DEFAULT_START_TIME),
            daily_end_time: UtcTimeValue(DEFAULT_END_TIME),
        }
    }
}
