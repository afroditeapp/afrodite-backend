use error_stack::{Result, ResultExt, report};
use image::RgbaImage;
use nsfw::{Model, model::Metric};
use simple_backend_config::file::{
    ImageProcessingConfig, NsfwDetectionConfig, NsfwDetectionThresholds,
};

use crate::ImageProcessError;

struct State {
    model: Model,
    config: NsfwDetectionConfig,
}

pub struct NsfwDetector {
    state: Option<State>,
}

impl NsfwDetector {
    pub fn new(config: &ImageProcessingConfig) -> Result<Self, ImageProcessError> {
        let Some(config) = config.nsfw_detection.clone() else {
            return Ok(Self { state: None });
        };

        let file = std::fs::File::open(&config.model_file)
            .change_context(ImageProcessError::NsfwDetectionError)?;
        let model = nsfw::create_model(file).map_err(|e| {
            report!(ImageProcessError::NsfwDetectionError).attach_printable(e.to_string())
        })?;

        Ok(Self {
            state: Some(State { model, config }),
        })
    }

    pub fn detect_nsfw(&self, img: RgbaImage) -> Result<bool, ImageProcessError> {
        let Some(state) = &self.state else {
            return Ok(false);
        };

        let results = nsfw::examine(&state.model, &img).map_err(|e| {
            report!(ImageProcessError::NsfwDetectionError).attach_printable(e.to_string())
        })?;

        if state.config.debug_log_results() {
            eprintln!("NSFW detection results: {results:?}");
        }

        fn threshold(m: &Metric, thresholds: &NsfwDetectionThresholds) -> Option<f32> {
            match m {
                Metric::Drawings => thresholds.drawings,
                Metric::Hentai => thresholds.hentai,
                Metric::Neutral => thresholds.neutral,
                Metric::Porn => thresholds.porn,
                Metric::Sexy => thresholds.sexy,
            }
        }

        for c in &results {
            if let Some(threshold) = threshold(&c.metric, &state.config.thresholds) {
                if c.score >= threshold {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}
