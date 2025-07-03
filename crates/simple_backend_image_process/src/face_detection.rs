use error_stack::{Result, ResultExt, report};
use image::GrayImage;
use rustface::Model;
use simple_backend_config::file::{ImageProcessingConfig, SeetaFaceConfig};

use crate::ImageProcessError;

struct State {
    model: Model,
    config: SeetaFaceConfig,
}

pub struct FaceDetector {
    state: Option<State>,
}

impl FaceDetector {
    pub fn new(config: &ImageProcessingConfig) -> Result<Self, ImageProcessError> {
        let Some(config) = config.seetaface.clone() else {
            return Ok(Self { state: None });
        };

        let model = rustface::load_model(&config.model_file)
            .change_context(ImageProcessError::FaceDetection)?;

        Ok(Self {
            state: Some(State { model, config }),
        })
    }

    pub fn detect_face(&self, data: GrayImage) -> Result<bool, ImageProcessError> {
        let Some(state) = &self.state else {
            return Ok(false);
        };

        let data = rustface::ImageData::new(&data, data.width(), data.height());

        let result =
            std::panic::catch_unwind(|| -> Result<Vec<rustface::FaceInfo>, ImageProcessError> {
                let mut model = rustface::create_detector_with_model(state.model.clone());
                model.set_score_thresh(state.config.detection_threshold);
                model.set_pyramid_scale_factor(state.config.pyramid_scale_factor);
                model.set_min_face_size(20);
                model.set_slide_window_step(4, 4);

                Ok(model.detect(&data))
            });

        let data = match result {
            Ok(result) => result,
            Err(e) => {
                let error = e
                    .downcast_ref::<&str>()
                    .map(|message| message.to_string())
                    .unwrap_or_default();
                return Err(report!(ImageProcessError::FaceDetectionPanic).attach_printable(error));
            }
        }
        .change_context(ImageProcessError::FaceDetection)?;

        if state.config.debug_log_results() {
            eprintln!("Face detection results: {data:?}");
        }

        Ok(!data.is_empty())
    }
}
