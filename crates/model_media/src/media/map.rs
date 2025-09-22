use serde::{Deserialize, Serialize};
use utoipa::IntoParams;

/// Y coordinate of slippy map tile.
///
/// This might include also .png file extension.
#[derive(Debug, Clone, Deserialize, Serialize, IntoParams)]
pub struct MapTileY {
    pub y: String,
}

/// X coordinate of slippy map tile.
#[derive(Debug, Clone, Deserialize, Serialize, IntoParams)]
pub struct MapTileX {
    pub x: u32,
}

/// Z coordinate (or zoom number) of slippy map tile.
#[derive(Debug, Clone, Deserialize, Serialize, IntoParams)]
pub struct MapTileZ {
    pub z: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, IntoParams)]
pub struct MapTileVersion {
    pub v: u32,
}
