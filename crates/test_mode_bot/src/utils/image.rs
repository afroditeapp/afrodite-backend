use std::path::{Path, PathBuf};

use image::{ImageBuffer, Rgb, codecs::jpeg::JpegEncoder};
use rand::seq::SliceRandom;

const GENERATED_IMG_WIDTH: u32 = 512;
const GENERATED_IMG_HEIGHT: u32 = 512;

pub struct ImageProvider {}

impl ImageProvider {
    pub fn jpeg_image_with_color(color: [u8; 3]) -> Vec<u8> {
        let mut buffer: ImageBuffer<Rgb<u8>, _> =
            image::ImageBuffer::new(GENERATED_IMG_WIDTH, GENERATED_IMG_HEIGHT);

        for pixel in buffer.pixels_mut() {
            pixel.0 = color;
        }

        let mut data = vec![];
        let mut encoder = JpegEncoder::new(&mut data);
        encoder.encode_image(&buffer).unwrap();

        data
    }

    pub fn random_jpeg_image() -> Vec<u8> {
        let img_color = rand::random();
        Self::jpeg_image_with_color(img_color)
    }

    pub fn default_jpeg_image() -> Vec<u8> {
        let img_color = [0, 145, 255];
        Self::jpeg_image_with_color(img_color)
    }

    pub fn random_image_from_directory(dir: &Path) -> Result<Option<PathBuf>, std::io::Error> {
        let mut imgs = vec![];
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.as_os_str().to_string_lossy();
            if name.ends_with("jpeg") || name.ends_with("jpg") {
                imgs.push(entry.path());
            }
        }

        Ok(imgs
            .choose(&mut rand::thread_rng())
            .map(|path| path.to_owned()))
    }

    pub fn mark_jpeg_image(jpeg_img: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        image::load_from_memory_with_format(jpeg_img, image::ImageFormat::Jpeg)
            .and_then(|img| {
                let mut img = img.into_rgb8();
                let mark_height = ((img.height() as f64) * 0.1) as usize;

                img.rows_mut().take(mark_height).for_each(|row| {
                    row.for_each(|pixel| {
                        pixel[0] = 0;
                        pixel[1] = 145;
                        pixel[2] = 255;
                    })
                });

                img.rows_mut().rev().take(mark_height).for_each(|row| {
                    row.for_each(|pixel| {
                        pixel[0] = 0;
                        pixel[1] = 145;
                        pixel[2] = 255;
                    })
                });

                let mut data = vec![];
                let mut encoder = JpegEncoder::new(&mut data);
                encoder.encode_image(&img).map(|_| data)
            })
            .map_err(std::io::Error::other)
    }
}
