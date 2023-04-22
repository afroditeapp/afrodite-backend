use image::{ImageBuffer, Rgb, Rgba, codecs::jpeg::JpegEncoder};




pub struct ImageProvider {

}

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

        for pixel in buffer.pixels_mut() {
            pixel.0 = rand::random();
        }

        let mut data = vec![];
        let mut encoder = JpegEncoder::new(&mut data);
        encoder.encode_image(&buffer).unwrap();

        data
    }
}
