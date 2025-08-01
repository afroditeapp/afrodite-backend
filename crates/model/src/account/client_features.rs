use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use simple_backend_utils::time::UtcTimeValue;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ClientFeaturesFileHash {
    h: String,
}

impl ClientFeaturesFileHash {
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
    pub news: Option<NewsConfig>,
    #[serde(default)]
    pub map: MapConfigInternal,
    #[serde(default)]
    pub limits: LimitsConfigInternal,
}

impl ClientFeaturesConfigInternal {
    pub fn to_client_features_config(self) -> Result<ClientFeaturesConfig, String> {
        if let Some(key) = self.attribution.other.keys().next() {
            return Err(format!(
                "Client features config file error. Unknown attribution string resource '{key}'."
            ));
        }

        Ok(ClientFeaturesConfig {
            attribution: self.attribution.into(),
            features: self.features,
            news: self.news,
            map: self.map.into(),
            limits: self.limits.into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ClientFeaturesConfig {
    pub attribution: AttributionConfig,
    pub features: FeaturesConfig,
    /// Enable news UI
    pub news: Option<NewsConfig>,
    pub map: MapConfig,
    pub limits: LimitsConfig,
}

impl ClientFeaturesConfig {
    pub fn daily_likes(&self) -> Option<i64> {
        self.limits
            .likes
            .like_sending
            .as_ref()
            .map(|v| v.daily_limit.into())
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
    pub generic: Option<StringResource>,
    /// Attribution info text displayed when IP country data is shown.
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
    /// Make possible for admins to write translations for news.
    /// If news translation is not available then server returns
    /// news with locale "default".
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
}

#[derive(Debug, Default, Clone, Serialize, ToSchema)]
pub struct MapConfig {
    /// Limit viewable map area
    pub bounds: MapBounds,
    pub zoom: MapZoom,
    pub initial_location: MapCoordinate,
}

impl From<MapConfigInternal> for MapConfig {
    fn from(value: MapConfigInternal) -> Self {
        Self {
            bounds: value.bounds,
            zoom: value.zoom,
            initial_location: value.initial_location,
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
    pub unlimited_likes_disabling_time: Option<UtcTimeValue>,
    pub like_sending: Option<LikeSendingLimitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LikeSendingLimitConfig {
    pub daily_limit: u8,
    /// UTC time with "hh:mm" format.
    pub reset_time: UtcTimeValue,
}
