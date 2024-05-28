use std::{
    io::BufReader,
    path::{Path, PathBuf},
};

use error_stack::{report, Result, ResultExt};
use image::{DynamicImage, EncodableLayout};
use simple_backend_config::args::InputFileType;

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
}

pub struct Settings {
    /// Input image file.
    pub input: PathBuf,
    pub input_file_type: InputFileType,
    /// Output jpeg image file. Will be overwritten if exists.
    pub output: PathBuf,
    /// Output jpeg image quality. Clamped to 1-100 range.
    /// Mozjpeg library recommends values in 60-80 range.
    pub quality: f32,
}

pub fn handle_image(settings: Settings) -> Result<(), ImageProcessError> {
    let format = match settings.input_file_type {
        InputFileType::JpegImage => image::ImageFormat::Jpeg,
    };

    // Only JPEG images are supported
    let rotation = read_exif_rotation_info(&settings.input).unwrap_or(0);

    let img_file = std::fs::File::open(&settings.input)
        .change_context(ImageProcessError::InputReadingFailed)?;
    let buffered_reader = BufReader::new(img_file);
    let img = image::io::Reader::with_format(buffered_reader, format)
        .decode()
        .change_context(ImageProcessError::InputReadingFailed)?;

    if img.width() < SOURCE_IMG_MIN_WIDTH_AND_HEIGHT
        || img.height() < SOURCE_IMG_MIN_WIDTH_AND_HEIGHT
    {
        return Err(report!(ImageProcessError::SourceImageTooSmall));
    }

    let img = resize_and_rotate_image(img, rotation);

    let result = std::panic::catch_unwind(|| -> Result<Vec<u8>, ImageProcessError> {
        let mut compress = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);

        compress.set_size(
            TryInto::<usize>::try_into(img.width())
                .change_context(ImageProcessError::EncodingError)?,
            TryInto::<usize>::try_into(img.height())
                .change_context(ImageProcessError::EncodingError)?,
        );

        let quality = settings.quality.clamp(1.0, 100.0);
        let quality = if quality.is_nan() { 1.0 } else { quality };
        compress.set_quality(quality);

        let mut compress = compress
            .start_compress(Vec::new())
            .change_context(ImageProcessError::EncodingError)?;

        let data = img.into_rgb8();
        compress
            .write_scanlines(data.as_bytes())
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

    std::fs::write(&settings.output, data).change_context(ImageProcessError::FileWriting)?;

    Ok(())
}

/// Read exif rotation info from jpeg image.
/// Returns error if reading failed or the rotation info does not exists.
fn read_exif_rotation_info(image: &Path) -> Result<u32, ImageProcessError> {
    let file = std::fs::File::open(image).change_context(ImageProcessError::ExifReadingFailed)?;
    let mut buf_reader = std::io::BufReader::new(file);
    let reader = exif::Reader::new();
    let exif = reader
        .read_from_container(&mut buf_reader)
        .change_context(ImageProcessError::ExifReadingFailed)?;

    let field = exif
        .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .ok_or(report!(ImageProcessError::ExifReadingFailed))?;
    let value = field
        .value
        .get_uint(0)
        .ok_or(report!(ImageProcessError::ExifReadingFailed))?;

    Ok(value)
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

fn resize_and_rotate_image(img: DynamicImage, exif_rotation: u32) -> DynamicImage {
    let img = resize_image_if_needed(img);
    match exif_rotation {
        1 => img,
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}
