use core::models::io::{ImageWriter, ImageIOError, ImageWriterOptions};
use core::models::image::Image;
pub struct PNGWriter {
}

impl PNGWriter {
    const fn new() -> Self {
        PNGWriter {}
    }
}

impl ImageWriter for PNGWriter {
    
    fn write(&self, _image: &Image, _options: &ImageWriterOptions) -> Result<Vec<u8>, ImageIOError> {
        Result::Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple_test() {
    }
}