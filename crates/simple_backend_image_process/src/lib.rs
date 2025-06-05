use std::io::Write;

use error_stack::{report, Result, ResultExt};
use image::{DynamicImage, EncodableLayout, GrayImage, ImageDecoder, ImageReader};
use nsfw_detection::handle_nsfw_detection;
use serde::{Deserialize, Serialize};
use simple_backend_config::{
    args::{ImageProcessModeArgs, InputFileType},
    file::ImageProcessingConfig,
};

mod nsfw_detection;

const SOURCE_IMG_MIN_WIDTH_AND_HEIGHT: u32 = 512;

#[derive(thiserror::Error, Debug)]
pub enum ImageProcessError {
    #[error("Input reading failed")]
    InputReadingFailed,

    #[error("Mozjpeg panic detected")]
    MozjpegPanic,

    #[error("Encoding error detected")]
    EncodingError,

    #[error("File writing failed")]
    FileWriting,

    #[error("Exif reading failed")]
    ExifReadingFailed,

    #[error(
        "Source image width or height is less than {}",
        SOURCE_IMG_MIN_WIDTH_AND_HEIGHT
    )]
    SourceImageTooSmall,

    #[error("Face detection error")]
    FaceDetection,

    #[error("Face detection panic detect")]
    FaceDetectionPanic,

    #[error("Stdout error")]
    Stdout,

    #[error("NSFW detection error")]
    NsfwDetectionError,
}

/// Image process returns this info as JSON to standard output.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ImageProcessingInfo {
    pub face_detected: bool,
    pub nsfw_detected: bool,
}

pub fn handle_image(
    args: ImageProcessModeArgs,
    config: ImageProcessingConfig,
) -> Result<(), ImageProcessError> {
    let format = match args.input_file_type {
        InputFileType::JpegImage => image::ImageFormat::Jpeg,
    };

    let mut img_reader =
        ImageReader::open(&args.input).change_context(ImageProcessError::InputReadingFailed)?;
    img_reader.set_format(format);
    let mut img_decoder = img_reader
        .into_decoder()
        .change_context(ImageProcessError::InputReadingFailed)?;
    let orientation = img_decoder
        .orientation()
        .change_context(ImageProcessError::ExifReadingFailed)?;
    let img = DynamicImage::from_decoder(img_decoder)
        .change_context(ImageProcessError::InputReadingFailed)?;

    if img.width() < SOURCE_IMG_MIN_WIDTH_AND_HEIGHT
        || img.height() < SOURCE_IMG_MIN_WIDTH_AND_HEIGHT
    {
        return Err(report!(ImageProcessError::SourceImageTooSmall));
    }

    let mut img = resize_image_if_needed(img);
    img.apply_orientation(orientation);
    let width = img.width();
    let height = img.height();

    let result = std::panic::catch_unwind(|| -> Result<Vec<u8>, ImageProcessError> {
        let mut compress = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);

        compress.set_size(
            TryInto::<usize>::try_into(width).change_context(ImageProcessError::EncodingError)?,
            TryInto::<usize>::try_into(height).change_context(ImageProcessError::EncodingError)?,
        );

        let quality = config.jpeg_quality().clamp(1.0, 100.0);
        let quality = if quality.is_nan() { 1.0 } else { quality };
        compress.set_quality(quality);

        let mut compress = compress
            .start_compress(Vec::new())
            .change_context(ImageProcessError::EncodingError)?;

        compress
            .write_scanlines(img.to_rgb8().as_bytes())
            .change_context(ImageProcessError::EncodingError)?;

        let data = compress
            .finish()
            .change_context(ImageProcessError::EncodingError)?;
        Ok(data)
    });

    let data = match result {
        Ok(result) => result,
        Err(e) => {
            let error = e
                .downcast_ref::<&str>()
                .map(|message| message.to_string())
                .unwrap_or_default();
            return Err(report!(ImageProcessError::MozjpegPanic).attach_printable(error));
        }
    }
    .change_context(ImageProcessError::EncodingError)?;

    std::fs::write(&args.output, data).change_context(ImageProcessError::FileWriting)?;

    let face_detected = match detect_face(&config, img.to_luma8()) {
        Ok(v) => v,
        Err(e) => {
            // Ignore
            eprintln!("{:?}", e);
            false
        }
    };

    let nsfw_detected = handle_nsfw_detection(&config, img.into_rgba8())?;

    let info = ImageProcessingInfo {
        face_detected,
        nsfw_detected,
    };

    let mut stdout = std::io::stdout();
    serde_json::to_writer(&stdout, &info).change_context(ImageProcessError::Stdout)?;
    stdout.flush().change_context(ImageProcessError::Stdout)?;

    Ok(())
}

fn resize_image_if_needed(img: DynamicImage) -> DynamicImage {
    const WIDTH: u32 = 1920;
    const HEIGHT: u32 = 1080;

    // Check both using width because it is larger value
    if img.width() > WIDTH || img.height() > WIDTH {
        // Resize, so that suggested new resolution matches the image
        // orientation. This makes resized image the largest possible which can
        // fit in Full HD area with the same aspect ratio.
        if img.width() > img.height() {
            img.resize(WIDTH, HEIGHT, image::imageops::FilterType::Lanczos3)
        } else {
            img.resize(HEIGHT, WIDTH, image::imageops::FilterType::Lanczos3)
        }
    } else {
        img
    }
}

fn detect_face(
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
