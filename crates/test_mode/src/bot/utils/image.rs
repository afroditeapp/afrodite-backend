use std::path::Path;

use image::{codecs::jpeg::JpegEncoder, ImageBuffer, Rgb};
use rand::seq::SliceRandom;

pub struct ImageProvider {}

impl ImageProvider {
    pub fn jpeg_image() -> Vec<u8> {
        let mut buffer: ImageBuffer<Rgb<u8>, _> = image::ImageBuffer::new(512, 512);

        for pixel in buffer.pixels_mut() {
            pixel.0 = [255, 255, 255];
        }

        let mut data = vec![];
        let mut encoder = JpegEncoder::new(&mut data);
        encoder.encode_image(&buffer).unwrap();

        data
    }

    pub fn random_jpeg_image() -> Vec<u8> {
        let mut buffer: ImageBuffer<Rgb<u8>, _> = image::ImageBuffer::new(512, 512);

        let img_color = rand::random();

        for pixel in buffer.pixels_mut() {
            pixel.0 = img_color;
        }

        let mut data = vec![];
        let mut encoder = JpegEncoder::new(&mut data);
        encoder.encode_image(&buffer).unwrap();

        data
    }

    pub fn random_image_from_directory(dir: &Path) -> Result<Option<Vec<u8>>, std::io::Error> {
        let mut imgs = vec![];
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.as_os_str().to_string_lossy();
            if name.ends_with("jpeg") || name.ends_with("jpg") {
                imgs.push(entry.path());
            }
        }

        if let Some(img) = imgs.choose(&mut rand::thread_rng()) {
            Ok(Some(std::fs::read(img)?))
        } else {
            Ok(None)
        }
    }
}
