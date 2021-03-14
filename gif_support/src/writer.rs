use core::models::{image::Image, io::{ImageIOError, ImageWriter, ImageWriterOptions}};

pub struct GIFWriter {
}

impl GIFWriter {

    pub fn new() -> Self {
        GIFWriter {
        }
    }
}

impl ImageWriter for GIFWriter {
    
    fn write(&self, _image: &Image, _options: &ImageWriterOptions) -> Result<Vec<u8>, ImageIOError> {
        todo!()
    }
}