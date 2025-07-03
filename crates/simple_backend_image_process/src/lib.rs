use std::{io, path::PathBuf};

use error_stack::{Result, ResultExt, report};
use face_detection::FaceDetector;
use image::{DynamicImage, EncodableLayout, ImageDecoder, ImageReader};
use nsfw_detection::NsfwDetector;
use serde::{Deserialize, Serialize};
use simple_backend_config::file::ImageProcessingConfig;

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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum InputFileType {
    JpegImage,
}

/// Image process reads this info as JSON from standard input.
///
/// The standard input receives JSON strings with this format
///
/// * String length (u32, little-endian)
/// * String bytes
///
/// The image process processs the JSON and responds with
/// writing [ImageProcessingInfo] to standard output.
#[derive(Debug, Deserialize, Serialize)]
pub struct ImageProcessingCommand {
    /// Input image file.
    pub input: PathBuf,
    pub input_file_type: InputFileType,
    /// Output jpeg image file. Will be overwritten if exists.
    pub output: PathBuf,
}

/// Image process returns this info as JSON to standard output.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ImageProcessingInfo {
    pub face_detected: bool,
    pub nsfw_detected: bool,
}

pub fn read_command(read: &mut impl io::Read) -> Result<ImageProcessingCommand, ImageProcessError> {
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

pub fn run_image_processing_loop(config: ImageProcessingConfig) -> Result<(), ImageProcessError> {
    let face_detector = FaceDetector::new(&config)?;
    let nsfw_detector = NsfwDetector::new(&config)?;

    let mut stdout = std::io::stdout();
    let mut stdin = std::io::stdin();

    loop {
        let command = read_command(&mut stdin)?;
        let info = handle_image(&config, &face_detector, &nsfw_detector, command)?;
        write_info(&mut stdout, info)?;
    }
}

fn handle_image(
    config: &ImageProcessingConfig,
    face_detector: &FaceDetector,
    nsfw_detector: &NsfwDetector,
    command: ImageProcessingCommand,
) -> Result<ImageProcessingInfo, ImageProcessError> {
    let format = match command.input_file_type {
        InputFileType::JpegImage => image::ImageFormat::Jpeg,
    };

    let mut img_reader =
        ImageReader::open(&command.input).change_context(ImageProcessError::InputReadingFailed)?;
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

    std::fs::write(&command.output, data).change_context(ImageProcessError::FileWriting)?;

    let face_detected = match face_detector.detect_face(img.to_luma8()) {
        Ok(v) => v,
        Err(e) => {
            // Ignore
            eprintln!("{e:?}");
            false
        }
    };

    let nsfw_detected = nsfw_detector.detect_nsfw(img.into_rgba8())?;

    let info = ImageProcessingInfo {
        face_detected,
        nsfw_detected,
    };

    Ok(info)
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
