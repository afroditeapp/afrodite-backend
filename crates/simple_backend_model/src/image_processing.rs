use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default)]
pub struct ImageProcessingDynamicConfig {
    /// See [rustface::Detector::set_score_thresh] documentation.
    /// Value 1.0 seems to work well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seetaface_threshold: Option<f64>,
    /// Thresholds when an image is classified as NSFW.
    ///
    /// If a probability value is equal or greater than the related
    /// threshold then the image is classified as NSFW.
    ///
    /// Threshold values must be in the range 0.0–1.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw_thresholds: Option<NsfwDetectionThresholds>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Default)]
pub struct NsfwDetectionThresholds {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drawings: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hentai: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neutral: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub porn: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sexy: Option<f64>,
}
