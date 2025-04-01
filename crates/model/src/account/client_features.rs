
use serde::{Deserialize, Serialize};
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
    pub features: FeaturesConfig,
    #[serde(default)]
    pub map: MapConfigInternal,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ClientFeaturesConfig {
    pub features: FeaturesConfig,
    pub map: MapConfig,
}

impl From<ClientFeaturesConfigInternal> for ClientFeaturesConfig {
    fn from(value: ClientFeaturesConfigInternal) -> Self {
        Self {
            features: value.features,
            map: value.map.into(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeaturesConfig {
    /// Enable news UI
    pub news: bool,
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
