use std::{io, path::PathBuf};

use error_stack::{IntoReport, ResultExt};
use face_detection::FaceDetector;
use image::{DynamicImage, EncodableLayout, ImageDecoder, ImageReader};
use nsfw_detection::NsfwDetector;
use serde::{Deserialize, Serialize};
use simple_backend_config::image_process::ImageProcessingConfig;
use simple_backend_utils::Result;

mod face_detection;
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

    #[error("Command reading failed")]
    ReadCommand,

    #[error("Info writing failed")]
    WriteInfo,

    #[error("NSFW detection error")]
    NsfwDetectionError,
}

/// Image process reads this info as JSON from standard input.
///
/// The standard input receives JSON strings with this format
///
/// * String length (u32, little-endian)
/// * String bytes
///
/// The image process processs the JSON and responds with
/// writing [ImageProcessingInfo] to standard output if the message is
/// [ImageProcessMessage::ProcessImage].
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "message_type")]
pub enum ImageProcessMessage {
    ProcessImage {
        process_image: ProcessImageCommand,
    },
    ChangeSettings {
        change_settings: ChangeSettingsCommand,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessImageCommand {
    /// Input image file.
    pub input: PathBuf,
    /// Output jpeg image file for high quality. Will be overwritten if exists.
    pub output_high: PathBuf,
    /// Output jpeg image file for medium quality. Will be overwritten if exists.
    pub output_medium: PathBuf,
    /// Output jpeg image file for low quality. Will be overwritten if exists.
    pub output_low: PathBuf,
    /// Output jpeg image file for lower quality. Will be overwritten if exists.
    pub output_lower: PathBuf,
    /// Output jpeg image file for very low quality. Will be overwritten if exists.
    pub output_very_low: PathBuf,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChangeSettingsCommand {
    pub settings: ImageProcessingConfig,
}

/// Image process returns this info as JSON to standard output.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ImageProcessingInfo {
    pub face_detected: bool,
    pub nsfw_detected: bool,
}

pub fn read_message(read: &mut impl io::Read) -> Result<ImageProcessMessage, ImageProcessError> {
    let mut length = [0; 4];
    read.read_exact(&mut length)
        .change_context(ImageProcessError::ReadCommand)?;
    let length = u32::from_le_bytes(length);
    let mut bytes: Vec<u8> = vec![0; length as usize];
    read.read_exact(&mut bytes)
        .change_context(ImageProcessError::ReadCommand)?;
    serde_json::from_reader(bytes.as_slice()).change_context(ImageProcessError::ReadCommand)
}

pub fn write_info(
    write: &mut impl io::Write,
    info: ImageProcessingInfo,
) -> Result<(), ImageProcessError> {
    let string = serde_json::to_string(&info).change_context(ImageProcessError::WriteInfo)?;
    let len =
        TryInto::<u32>::try_into(string.len()).change_context(ImageProcessError::WriteInfo)?;
    write
        .write_all(&len.to_le_bytes())
        .change_context(ImageProcessError::WriteInfo)?;
    write
        .write_all(string.as_bytes())
        .change_context(ImageProcessError::WriteInfo)?;
    write.flush().change_context(ImageProcessError::WriteInfo)?;
    Ok(())
}

pub fn run_image_processing_loop() -> Result<(), ImageProcessError> {
    let mut stdout = std::io::stdout();
    let mut stdin = std::io::stdin();

    // Wait for initial settings message from stdin
    let message = read_message(&mut stdin)?;
    let mut config = match message {
        ImageProcessMessage::ChangeSettings { change_settings } => change_settings.settings,
        ImageProcessMessage::ProcessImage { .. } => {
            return Err(ImageProcessError::ReadCommand
                .into_report()
                .attach("Expected initial ChangeSettings message, got ProcessImage"));
        }
    };

    let mut face_detector = FaceDetector::new(&config)?;
    let mut nsfw_detector = NsfwDetector::new(&config)?;

    loop {
        let message = read_message(&mut stdin)?;

        match message {
            ImageProcessMessage::ProcessImage { process_image } => {
                let info = handle_image(&config, &face_detector, &nsfw_detector, process_image)?;
                write_info(&mut stdout, info)?;
            }
            ImageProcessMessage::ChangeSettings { change_settings } => {
                config = change_settings.settings;
                face_detector = FaceDetector::new(&config)?;
                nsfw_detector = NsfwDetector::new(&config)?;
            }
        }
    }
}

fn handle_image(
    config: &ImageProcessingConfig,
    face_detector: &FaceDetector,
    nsfw_detector: &NsfwDetector,
    command: ProcessImageCommand,
) -> Result<ImageProcessingInfo, ImageProcessError> {
    let mut img_decoder = ImageReader::open(&command.input)
        .change_context(ImageProcessError::InputReadingFailed)?
        .with_guessed_format()
        .change_context(ImageProcessError::InputReadingFailed)?
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
        return Err(ImageProcessError::SourceImageTooSmall.into_report());
    }

    let mut oriented = img;
    oriented.apply_orientation(orientation);

    let high = resize_image_if_needed(&oriented, 1280);

    let face_detected = match face_detector.detect_face(high.to_luma8()) {
        Ok(v) => v,
        Err(e) => {
            // Ignore
            eprintln!("{e:?}");
            false
        }
    };

    let nsfw_detected = nsfw_detector.detect_nsfw(high.to_rgba8())?;

    encode_and_save_jpeg(config, &high, &command.output_high)?;

    let medium = resize_image_if_needed(&oriented, 854);
    encode_and_save_jpeg(config, &medium, &command.output_medium)?;

    let low = resize_image_if_needed(&oriented, 640);
    encode_and_save_jpeg(config, &low, &command.output_low)?;

    let lower = resize_image_if_needed(&oriented, 426);
    encode_and_save_jpeg(config, &lower, &command.output_lower)?;

    let very_low = resize_image_if_needed(&oriented, 256);
    encode_and_save_jpeg(config, &very_low, &command.output_very_low)?;

    let info = ImageProcessingInfo {
        face_detected,
        nsfw_detected,
    };

    Ok(info)
}

fn encode_and_save_jpeg(
    config: &ImageProcessingConfig,
    img: &DynamicImage,
    output_path: &PathBuf,
) -> Result<(), ImageProcessError> {
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
            return Err(ImageProcessError::MozjpegPanic.into_report().attach(error));
        }
    }
    .change_context(ImageProcessError::EncodingError)?;

    std::fs::write(output_path, data).change_context(ImageProcessError::FileWriting)?;
    Ok(())
}

fn resize_image_if_needed(img: &DynamicImage, size: u32) -> DynamicImage {
    if img.width() > size || img.height() > size {
        img.resize(size, size, image::imageops::FilterType::Lanczos3)
    } else {
        img.clone()
    }
}
