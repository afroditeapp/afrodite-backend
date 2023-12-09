//! Slippy map tile server logic.

use simple_backend_config::{SimpleBackendConfig, file::TileMapConfig};
use error_stack::{ResultExt, Result};

#[derive(thiserror::Error, Debug)]
pub enum TileMapError {
    #[error("File reading failed")]
    IoFileRead,

    #[error("Missing tile map config")]
    MissingTileMapConfig,
}

pub struct TileMapManager {
    config: Option<TileMapConfig>,
}

impl TileMapManager {
    pub fn new(config: &SimpleBackendConfig) -> Self {
        Self { config: config.tile_map().cloned() }
    }

    pub async fn load_map_tile(&self, z: u32, x: u32, y: u32) -> Result<Option<Vec<u8>>, TileMapError> {
        let config = self.config
            .as_ref()
            .ok_or(TileMapError::MissingTileMapConfig)?;

        let path = config.tile_dir.join(format!("{}/{}/{}.png", z, x, y));

        if !path.exists() {
            return Ok(None);
        }

        // TODO: tile map cache

        let data = tokio::fs::read(path)
            .await
            .change_context(TileMapError::IoFileRead)?;

        Ok(Some(data))
    }
}
