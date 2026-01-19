use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct ImageProcessingDynamicConfig {
    /// See [rustface::Detector::set_score_thresh] documentation.
    /// Value 1.0 seems to work well.
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
    pub drawings: Option<f64>,
    pub hentai: Option<f64>,
    pub neutral: Option<f64>,
    pub porn: Option<f64>,
    pub sexy: Option<f64>,
}
