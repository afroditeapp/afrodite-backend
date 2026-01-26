use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use simple_backend_utils::time::UtcTimeValue;
use utoipa::ToSchema;

mod internal;
pub use internal::*;

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

#[derive(Debug, Default, Clone, Serialize, ToSchema)]
pub struct ClientFeaturesConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    attribution: Option<AttributionConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    features: Option<FeaturesConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    news: Option<NewsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    map: Option<MapConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    likes: Option<LikesConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile: Option<ProfileConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat: Option<ChatConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    server: Option<ServerConfig>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct AttributionConfig {
    /// Generic attribution info text displayed in about screen
    /// of the app.
    #[serde(skip_serializing_if = "Option::is_none")]
    generic: Option<StringResource>,
    /// Attribution info text displayed when IP country data is shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    ip_country: Option<StringResource>,
}

impl From<AttributionConfigInternal> for AttributionConfig {
    fn from(value: AttributionConfigInternal) -> Self {
        Self {
            generic: value.generic.map(Into::into),
            ip_country: value.ip_country.map(Into::into),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct StringResource {
    default: String,
    /// Keys are country codes like "en".
    translations: HashMap<String, String>,
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
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    video_calls: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewsConfig {
    /// Make possible for admins to write translations for news.
    /// If news translation is not available then server returns
    /// news with locale "default".
    locales: Vec<String>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct MapConfig {
    /// Limit viewable map area
    #[serde(skip_serializing_if = "Option::is_none")]
    bounds: Option<MapBounds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    zoom: Option<MapZoom>,
    #[serde(skip_serializing_if = "Option::is_none")]
    initial_location: Option<MapCoordinate>,
    /// Increase this version number to make client to redownload cached
    /// map tiles.
    #[serde(default, skip_serializing_if = "is_zero")]
    #[schema(default = 0)]
    pub tile_data_version: u32,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapBounds {
    top_left: MapCoordinate,
    bottom_right: MapCoordinate,
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
    lat: f64,
    /// Longitude
    lon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MapZoom {
    min: u8,
    max: u8,
    max_tile_downloading: u8,
    location_not_selected: u8,
    location_selected: u8,
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct LikesConfig {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_image: Option<FirstImageConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct FirstImageConfig {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub require_face_detected_when_editing: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub require_face_detected_when_viewing: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct ChatConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typing_indicator: Option<TypingIndicatorConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_online_status: Option<CheckOnlineStatusConfig>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub message_state_seen: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct TypingIndicatorConfig {
    /// Client should hide typing indicator after this time elapses
    /// from [crate::EventType::TypingStart].
    start_event_ttl_seconds: u16,
    /// Server ignores messages that are received before
    /// wait time elapses.
    pub min_wait_seconds_between_requests_server: u16,
    /// Client should wait at least this time before sending
    /// another typing indicator message.
    pub min_wait_seconds_between_requests_client: u16,
}

impl Default for TypingIndicatorConfig {
    fn default() -> Self {
        Self {
            start_event_ttl_seconds: 10,
            min_wait_seconds_between_requests_server: 1,
            min_wait_seconds_between_requests_client: 4,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CheckOnlineStatusConfig {
    /// Server ignores check online status requests that are received before
    /// wait time elapses.
    pub min_wait_seconds_between_requests_server: u16,
    /// Client should wait at least this time before sending
    /// another check online status request.
    pub min_wait_seconds_between_requests_client: u16,
}

impl Default for CheckOnlineStatusConfig {
    fn default() -> Self {
        Self {
            min_wait_seconds_between_requests_server: 4,
            min_wait_seconds_between_requests_client: 8,
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct ServerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    scheduled_tasks: Option<ScheduledTasksConfig>,
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
