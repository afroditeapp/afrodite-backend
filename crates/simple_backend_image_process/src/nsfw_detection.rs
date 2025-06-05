use error_stack::{report, Result, ResultExt};
use image::RgbaImage;
use nsfw::model::Metric;
use simple_backend_config::file::{ImageProcessingConfig, NsfwDetectionThresholds};

use crate::ImageProcessError;

pub fn handle_nsfw_detection(
    config: &ImageProcessingConfig,
    img: RgbaImage,
) -> Result<bool, ImageProcessError> {
    let Some(config) = &config.nsfw_detection else {
        return Ok(false);
    };

    let file = std::fs::File::open(&config.model_file)
        .change_context(ImageProcessError::NsfwDetectionError)?;
    let model = nsfw::create_model(file)
        .map_err(|e| report!(ImageProcessError::NsfwDetectionError)
            .attach_printable(e.to_string())
        )?;

    let results = nsfw::examine(&model, &img)
        .map_err(|e| report!(ImageProcessError::NsfwDetectionError).attach_printable(e.to_string()))?;

    fn threshold(m: &Metric, thresholds: &NsfwDetectionThresholds) -> Option<f32> {
        match m {
            Metric::Drawings => thresholds.drawings,
            Metric::Hentai => thresholds.hentai,
            Metric::Neutral => thresholds.neutral,
            Metric::Porn => thresholds.porn,
            Metric::Sexy => thresholds.sexy,
        }
    }

    if let Some(thresholds) = &config.reject {
        for c in &results {
            if let Some(threshold) = threshold(&c.metric, thresholds) {
                if c.score >= threshold {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}
