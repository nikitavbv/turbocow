use core::models::{ImageWriter, Image, ImageIOError};
pub struct PPMWriter {
}

impl PPMWriter {
    const fn new() -> Self {
        PPMWriter {}
    }
}

impl ImageWriter for PPMWriter {
    
    fn write(&self, image: &Image) -> Result<Vec<u8>, ImageIOError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"P3");
        // 10 - LF
        bytes.push(10);
        bytes.extend_from_slice(image.width.to_string().as_bytes());
        bytes.push(10);
        bytes.extend_from_slice(image.height.to_string().as_bytes());
        bytes.push(10);
        bytes.extend_from_slice(b"255");
        bytes.push(10);
        let mut pixels = image.pixels.iter();
        loop {
            for _ in 0..image.width {
                if let Some(pixel) = pixels.next() {
                    bytes.extend_from_slice(pixel.red.to_string().as_bytes());
                    bytes.extend_from_slice(b" ");
                    bytes.extend_from_slice(pixel.green.to_string().as_bytes());
                    bytes.extend_from_slice(b" ");
                    bytes.extend_from_slice(pixel.blue.to_string().as_bytes());
                    bytes.extend_from_slice(b" ");
                } else {
                    bytes.pop();
                    return Result::Ok(bytes)
                }
            }
            bytes.push(10);
        }
    }
}

#[cfg(test)]
mod tests {
    use core::models::Pixel;
    use std::fs::write;
    use super::*;

    #[test]
    fn simple_test() {
        let pixels = vec![
            Pixel::from_rgb(21, 45, 78),
            Pixel::from_rgb(29, 1, 78),
            Pixel::from_rgb(48, 45, 224),
            Pixel::from_rgb(97, 45, 64),
            Pixel::from_rgb(158, 45, 19),
            Pixel::from_rgb(42, 45, 0),
            Pixel::from_rgb(129, 45, 234),
            Pixel::from_rgb(248, 40, 129),
            Pixel::from_rgb(176, 45, 2),
        ];
        let image = Image {
            width: 3,
            height: 3,
            pixels
        };
        let writer = PPMWriter::new();
        write(
            "assets/result.ppm",
            writer.write(&image).expect("Failed to write the image")
        ).expect("Failed to save image to the file");
    }
}