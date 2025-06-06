
use error_stack::{report, Result, ResultExt};
use image::GrayImage;
use simple_backend_config::file::ImageProcessingConfig;

use crate::ImageProcessError;

pub fn detect_face(
    config: &ImageProcessingConfig,
    data: GrayImage,
) -> Result<bool, ImageProcessError> {
    let Some(config) = &config.seetaface else {
        return Ok(false);
    };

    let data = rustface::ImageData::new(&data, data.width(), data.height());

    let result =
        std::panic::catch_unwind(|| -> Result<Vec<rustface::FaceInfo>, ImageProcessError> {
            let mut model = rustface::create_detector(&config.model_file)
                .change_context(ImageProcessError::FaceDetection)?;

            model.set_score_thresh(config.detection_threshold);
            model.set_pyramid_scale_factor(config.pyramid_scale_factor);
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

    Ok(!data.is_empty())
}
