use diesel::sql_types::SmallInt;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::SimpleDieselEnum;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Deserialize,
    Serialize,
    ToSchema,
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
    TryFromPrimitive,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
pub enum SourceImageFileType {
    Jpeg = 0,
    Png = 1,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SourceImageInfo {
    /// SHA256 hash of the source image file bytes (32 bytes).
    pub source_image_sha256_hash: Vec<u8>,
    pub source_image_file_size_bytes: i64,
    pub source_image_width: i16,
    pub source_image_height: i16,
    pub source_image_file_type: SourceImageFileType,
}

impl SourceImageInfo {
    pub fn new(
        source_image_sha256_hash: Vec<u8>,
        source_image_file_size_bytes: i64,
        source_image_width: i16,
        source_image_height: i16,
        source_image_file_type: SourceImageFileType,
    ) -> Self {
        Self {
            source_image_sha256_hash,
            source_image_file_size_bytes,
            source_image_width,
            source_image_height,
            source_image_file_type,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct ImageProcessingDynamicConfig {
    /// See [rustface::Detector::set_score_thresh] documentation.
    /// Value 1.0 seems to work well.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub seetaface_threshold: Option<f64>,
    /// Thresholds when an image is classified as NSFW.
    ///
    /// If a probability value is equal or greater than the related
    /// threshold then the image is classified as NSFW.
    ///
    /// Threshold values must be in the range 0.0–1.0.
    pub nsfw_thresholds: NsfwDetectionThresholds,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct NsfwDetectionThresholds {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub drawings: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub hentai: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub neutral: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub porn: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub sexy: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct ImageProcessingWarnings {
    pub seetaface_file_config_missing: bool,
    pub nsfw_detection_file_config_missing: bool,
}

impl NsfwDetectionThresholds {
    pub fn all_disabled(&self) -> bool {
        *self == Self::default()
    }
}
