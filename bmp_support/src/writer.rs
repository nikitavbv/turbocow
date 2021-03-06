use core::models::{ImageWriter, Image};
pub struct BMPWriter {
}

impl ImageWriter for BMPWriter {
    
    fn write(&self, _image: &Image) -> Vec<u8> {
        println!("writing bmp image");

        Vec::new()
    }
}