use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use simple_backend_utils::time::UtcTimeValue;
use utoipa::ToSchema;

use crate::profile::IconResource;

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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DynamicClientFeaturesConfigHash {
    h: String,
}

impl DynamicClientFeaturesConfigHash {
    pub fn from_json_string(json: &str) -> Self {
        Self {
            h: format!("{:x}", Sha256::digest(json.as_bytes())),
        }
    }

    pub fn hash(&self) -> &str {
        &self.h
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct DynamicClientFeaturesConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info_banners: Option<InfoBannersConfig>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct InfoBannersConfig {
    // Use BTreeMap so that serialization order is stable
    pub banners: BTreeMap<String, InfoBanner>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct InfoBannerUrlButton {
    pub text: StringResource,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub enum InfoBannerMode {
    /// Data: [InfoBanner::text]
    Text,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub enum PredefinedBanner {
    ServerMaintenance,
    AdminBotOffline,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct BannerPlatform {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub android: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub ios: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub web: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct BannerVisibility {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub profiles: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub likes: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub chats: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub menu: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub conversation: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct InfoBanner {
    /// Server increments this field when banner is changed.
    /// It wraps, so use "not equal" comparison when checking
    /// version changes.
    #[serde(default, skip_serializing_if = "is_zero")]
    #[schema(default = 0)]
    pub version: u32,
    pub mode: InfoBannerMode,
    pub platform: BannerPlatform,
    pub visibility: BannerVisibility,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_predefined_banner: Option<PredefinedBanner>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextInfoBanner>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct TextInfoBanner {
    pub body: StringResource,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub dismissible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_button: Option<InfoBannerUrlButton>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub icon: Option<IconResource>,
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
    age_verification: Option<AgeVerificationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    account_verification: Option<AccountVerificationConfig>,
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct StringResource {
    default: String,
    // Use BTreeMap so that serialization order is stable
    /// Keys are country codes like "en".
    translations: BTreeMap<String, String>,
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
    /// Show face verification status and filters
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    face_verification: bool,
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

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct AgeVerificationConfig {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    verify_during_initial_setup: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methods: Option<AgeVerificationMethodsConfig>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct AgeVerificationMethodsConfig {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub debug: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub eudi: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct AccountVerificationPlatforms {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub android: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub ios: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub web: bool,
}

impl AccountVerificationPlatforms {
    pub fn is_enabled_for(&self, client_type: crate::ClientType) -> bool {
        match client_type {
            crate::ClientType::Android => self.android,
            crate::ClientType::Ios => self.ios,
            crate::ClientType::Web => self.web,
            crate::ClientType::Bot => false,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct AccountVerificationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methods: Option<AccountVerificationMethodsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<AccountVerificationScopesConfig>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct AccountVerificationMethodsConfig {
    #[serde(default)]
    #[schema(default)]
    pub debug: AccountVerificationPlatforms,
    #[serde(default)]
    #[schema(default)]
    pub eudi: AccountVerificationPlatforms,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct AccountVerificationScopesConfig {
    #[serde(default)]
    #[schema(default)]
    pub security_content: AccountVerificationPlatforms,
    #[serde(default)]
    #[schema(default)]
    pub profile_age_range: AccountVerificationPlatforms,
    #[serde(default)]
    #[schema(default)]
    pub profile_name: AccountVerificationPlatforms,
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
