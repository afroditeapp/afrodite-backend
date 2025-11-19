use std::collections::HashMap;

use regex::Regex;
use serde::{Deserialize, Serialize};
use simple_backend_utils::time::UtcTimeValue;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClientFeaturesConfigHash {
    h: String,
}

impl ClientFeaturesConfigHash {
    pub fn new(h: String) -> Self {
        Self { h }
    }

    pub fn hash(&self) -> &str {
        &self.h
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ClientFeaturesConfigInternal {
    #[serde(default)]
    pub attribution: AttributionConfigInternal,
    #[serde(default)]
    pub features: FeaturesConfig,
    #[serde(default)]
    pub news: NewsConfig,
    #[serde(default)]
    pub map: MapConfigInternal,
    #[serde(default)]
    pub limits: LimitsConfigInternal,
    #[serde(default)]
    pub profile: ProfileConfig,
    #[serde(default)]
    pub chat: ChatConfig,
    #[serde(default)]
    pub server: ServerConfig,
}

impl ClientFeaturesConfigInternal {
    pub fn to_client_features_config(self) -> Result<ClientFeaturesConfig, String> {
        if let Some(key) = self.attribution.other.keys().next() {
            return Err(format!(
                "Client features config file error. Unknown attribution string resource '{key}'."
            ));
        }

        if let Some(regex) = &self.profile.profile_name_regex {
            if !regex.starts_with('^') {
                return Err("Profile name regex does not start with '^'".to_string());
            }
            if !regex.ends_with('$') {
                return Err("Profile name regex does not end with '$'".to_string());
            }
            Regex::new(regex).map_err(|v| v.to_string())?;
        }

        Ok(ClientFeaturesConfig {
            attribution: self.attribution.into(),
            features: self.features,
            news: self.news,
            map: self.map.into(),
            limits: self.limits.into(),
            profile: self.profile,
            chat: self.chat,
            server: self.server,
        })
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ClientFeaturesConfig {
    pub attribution: AttributionConfig,
    pub features: FeaturesConfig,
    pub news: NewsConfig,
    pub map: MapConfig,
    pub limits: LimitsConfig,
    pub profile: ProfileConfig,
    pub chat: ChatConfig,
    pub server: ServerConfig,
}

impl ClientFeaturesConfig {
    pub fn daily_likes(&self) -> Option<i16> {
        self.limits
            .likes
            .daily
            .as_ref()
            .map(|v| v.daily_likes.into())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AttributionConfigInternal {
    pub generic: Option<StringResourceInternal>,
    pub ip_country: Option<StringResourceInternal>,
    #[serde(flatten)]
    pub other: toml::Table,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttributionConfig {
    /// Generic attribution info text displayed in about screen
    /// of the app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generic: Option<StringResource>,
    /// Attribution info text displayed when IP country data is shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_country: Option<StringResource>,
}

impl From<AttributionConfigInternal> for AttributionConfig {
    fn from(value: AttributionConfigInternal) -> Self {
        Self {
            generic: value.generic.map(Into::into),
            ip_country: value.ip_country.map(Into::into),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StringResourceInternal {
    pub default: String,
    /// Keys are country codes like "en".
    #[serde(flatten)]
    pub translations: HashMap<String, String>,
}

impl StringResourceInternal {
    pub fn values(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.default.as_str()).chain(self.translations.values().map(|v| v.as_str()))
    }

    pub fn all_strings_contain(&self, text: &str) -> bool {
        for v in self.values() {
            if !v.contains(text) {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct StringResource {
    pub default: String,
    /// Keys are country codes like "en".
    pub translations: HashMap<String, String>,
}

impl From<StringResourceInternal> for StringResource {
    fn from(value: StringResourceInternal) -> Self {
        Self {
            default: value.default,
            translations: value.translations,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeaturesConfig {
    /// Enable video calls
    pub video_calls: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewsConfig {
    pub enabled: bool,
    /// Make possible for admins to write translations for news.
    /// If news translation is not available then server returns
    /// news with locale "default".
    #[serde(default)]
    pub locales: Vec<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct MapConfigInternal {
    /// Limit viewable map area
    #[serde(default)]
    pub bounds: MapBounds,
    #[serde(default)]
    pub zoom: MapZoom,
    #[serde(default)]
    pub initial_location: MapCoordinate,
    #[serde(default)]
    pub tile_data_version: u32,
}

#[derive(Debug, Default, Clone, Serialize, ToSchema)]
pub struct MapConfig {
    /// Limit viewable map area
    pub bounds: MapBounds,
    pub zoom: MapZoom,
    pub initial_location: MapCoordinate,
    /// Increase this version number to make client to redownload cached
    /// map tiles.
    pub tile_data_version: u32,
}

impl From<MapConfigInternal> for MapConfig {
    fn from(value: MapConfigInternal) -> Self {
        Self {
            bounds: value.bounds,
            zoom: value.zoom,
            initial_location: value.initial_location,
            tile_data_version: value.tile_data_version,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapBounds {
    pub top_left: MapCoordinate,
    pub bottom_right: MapCoordinate,
}

impl Default for MapBounds {
    fn default() -> Self {
        Self {
            top_left: MapCoordinate {
                lat: 90.0,
                lon: -180.0,
            },
            bottom_right: MapCoordinate {
                lat: -90.0,
                lon: 180.0,
            },
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapCoordinate {
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapZoom {
    pub min: u8,
    pub max: u8,
    pub max_tile_downloading: u8,
    pub location_not_selected: u8,
    pub location_selected: u8,
}

impl Default for MapZoom {
    fn default() -> Self {
        Self {
            min: 0,
            max: 19,
            max_tile_downloading: 19,
            location_not_selected: 0,
            location_selected: 0,
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct LimitsConfigInternal {
    #[serde(default)]
    pub likes: LikeLimitsConfig,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct LimitsConfig {
    pub likes: LikeLimitsConfig,
}

impl From<LimitsConfigInternal> for LimitsConfig {
    fn from(value: LimitsConfigInternal) -> Self {
        Self { likes: value.likes }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct LikeLimitsConfig {
    /// UTC time with "hh:mm" format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlimited_likes_disabling_time: Option<UtcTimeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily: Option<DailyLikesConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DailyLikesConfig {
    pub daily_likes: u8,
    /// UTC time with "hh:mm" format.
    pub reset_time: UtcTimeValue,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProfileConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_name_regex: Option<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct ChatConfig {
    #[serde(default)]
    pub typing_indicator: TypingIndicatorConfig,
    #[serde(default)]
    pub check_online_status: CheckOnlineStatusConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct TypingIndicatorConfig {
    pub enabled: bool,
    /// Client should hide typing indicator after this time elapses
    /// from [crate::EventType::TypingStart].
    #[serde(default = "start_event_ttl_seconds_default")]
    start_event_ttl_seconds: u16,
    /// Server ignores messages that are received before
    /// wait time elapses.
    #[serde(default = "min_wait_seconds_between_sending_messages_default")]
    pub min_wait_seconds_between_sending_messages: u16,
}

fn start_event_ttl_seconds_default() -> u16 {
    10
}

fn min_wait_seconds_between_sending_messages_default() -> u16 {
    2
}

impl Default for TypingIndicatorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            start_event_ttl_seconds: start_event_ttl_seconds_default(),
            min_wait_seconds_between_sending_messages:
                min_wait_seconds_between_sending_messages_default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CheckOnlineStatusConfig {
    pub enabled: bool,
    /// After daily limit is reached, server ignores received check
    /// online status requests.
    #[serde(default = "daily_max_count_default")]
    pub daily_max_count: u16,
}

fn daily_max_count_default() -> u16 {
    100
}

impl Default for CheckOnlineStatusConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            daily_max_count: daily_max_count_default(),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct ServerConfig {
    #[serde(default)]
    pub scheduled_tasks: ScheduledTasksConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ScheduledTasksConfig {
    pub daily_start_time: UtcTimeValue,
}

impl Default for ScheduledTasksConfig {
    fn default() -> Self {
        use simple_backend_utils::time::TimeValue;
        const DEFAULT_SCHEDULED_TASKS_TIME: TimeValue = TimeValue::new(3, 0);

        Self {
            daily_start_time: UtcTimeValue(DEFAULT_SCHEDULED_TASKS_TIME),
        }
    }
}
