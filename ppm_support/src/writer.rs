use core::models::{image::Image, io::ImageIOError, io::{ImageWriter, ImageWriterOptions}};
pub struct PPMWriter {
}

impl PPMWriter {
    const fn new() -> Self {
        PPMWriter {}
    }
}

impl ImageWriter for PPMWriter {
    
    fn write(&self, image: &Image, _options: &ImageWriterOptions) -> Result<Vec<u8>, ImageIOError> {
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
    use core::models::pixel::Pixel;
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
        let data = writer.write(&image, &ImageWriterOptions::default())
            .expect("Failed to write test image");
        assert_eq!(std::str::from_utf8(&data).unwrap(),
"P3
3
3
255
21 45 78 29 1 78 48 45 224 
97 45 64 158 45 19 42 45 0 
129 45 234 248 40 129 176 45 2 ");
    }
}