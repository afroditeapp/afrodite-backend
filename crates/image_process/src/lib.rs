use std::path::PathBuf;

use error_stack::{Result, ResultExt, report};
use image::EncodableLayout;

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
}

pub struct Settings {
    /// Input image file. Only jpeg is supported currently.
    pub input: PathBuf,
    /// Output jpeg image file. Will be overwritten if exists.
    pub output: PathBuf,
    /// Output jpeg image quality. Clamped to 1-100 range.
    /// Mozjpeg library recommends values in 60-80 range.
    pub quality: f32,
}

pub fn handle_image(settings: Settings) -> Result<(), ImageProcessError> {
    let img = image::open(settings.input)
        .change_context(ImageProcessError::InputReadingFailed)?;

    let result = std::panic::catch_unwind(|| -> Result<Vec<u8>, ImageProcessError> {
        let mut compress = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);

        compress.set_size(
            TryInto::<usize>::try_into(img.width())
                .change_context(ImageProcessError::EncodingError)?,
            TryInto::<usize>::try_into(img.height())
                .change_context(ImageProcessError::EncodingError)?,
        );

        let quality = settings.quality.clamp(1.0, 100.0);
        let quality = if quality.is_nan() {
            1.0
        } else {
            quality
        };
        compress.set_quality(quality);

        let mut compress = compress.start_compress(Vec::new())
            .change_context(ImageProcessError::EncodingError)?;

        let data = img.to_rgb8();
        compress.write_scanlines(data.as_bytes())
            .change_context(ImageProcessError::EncodingError)?;

        let data = compress.finish().change_context(ImageProcessError::EncodingError)?;
        Ok(data)
    });

    let data = match result {
        Ok(result) => result,
        Err(e) => {
            let error = e.downcast_ref::<&str>()
                .map(|message| message.to_string())
                .unwrap_or_default();
            return Err(report!(ImageProcessError::MozjpegPanic)
                .attach_printable(error));
        }
    }
        .change_context(ImageProcessError::EncodingError)?;

    std::fs::write(&settings.output, &data)
        .change_context(ImageProcessError::FileWriting)?;


    Ok(())
}