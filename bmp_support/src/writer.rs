use core::models::{Image, ImageIOError, ImageWriter};
pub struct BMPWriter {
}

impl ImageWriter for BMPWriter {
    
    fn write(&self, _image: &Image) -> Result<Vec<u8>, ImageIOError> {
        println!("writing bmp image");
        Ok(Vec::new())
    }
}