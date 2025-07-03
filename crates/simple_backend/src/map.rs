//! Slippy map tile server logic.

use error_stack::{Result, ResultExt};
use simple_backend_config::{SimpleBackendConfig, file::TileMapConfig};
use tokio_util::io::ReaderStream;

#[derive(thiserror::Error, Debug)]
pub enum TileMapError {
    #[error("File open failed")]
    IoFileOpen,
    #[error("Getting file metadata failed")]
    IoFileMetadata,

    #[error("Missing tile map config")]
    MissingTileMapConfig,
}

pub struct TileMapManager {
    config: Option<TileMapConfig>,
}

impl TileMapManager {
    pub fn new(config: &SimpleBackendConfig) -> Self {
        Self {
            config: config.tile_map().cloned(),
        }
    }

    pub async fn map_tile_byte_count_and_byte_stream(
        &self,
        z: u32,
        x: u32,
        y: u32,
    ) -> Result<Option<(u64, ReaderStream<tokio::fs::File>)>, TileMapError> {
        let config = self
            .config
            .as_ref()
            .ok_or(TileMapError::MissingTileMapConfig)?;

        let path = config.tile_dir.join(format!("{z}/{x}/{y}.png"));

        if !path.exists() {
            return Ok(None);
        }

        let file = tokio::fs::File::open(path)
            .await
            .change_context(TileMapError::IoFileOpen)?;
        let metadata = file
            .metadata()
            .await
            .change_context(TileMapError::IoFileMetadata)?;
        let stream = ReaderStream::new(file);

        Ok(Some((metadata.len(), stream)))
    }
}
