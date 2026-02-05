use serde::{Deserialize, Serialize};
use simple_backend_model::ImageProcessingDynamicConfig;

use crate::file::ImageProcessingStaticConfig;

/// Image processing configuration for the image process.
/// This type merges static config from config file and dynamic
/// config from DB.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageProcessingConfig {
    pub(crate) file: ImageProcessingStaticConfig,
    pub(crate) dynamic: ImageProcessingDynamicConfig,
}

impl ImageProcessingConfig {
    pub fn new(file: ImageProcessingStaticConfig, dynamic: ImageProcessingDynamicConfig) -> Self {
        Self { file, dynamic }
    }

    pub fn file(&self) -> &ImageProcessingStaticConfig {
        &self.file
    }

    pub fn dynamic(&self) -> &ImageProcessingDynamicConfig {
        &self.dynamic
    }

    pub fn jpeg_quality(&self) -> f32 {
        self.file.jpeg_quality as f32
    }
}
