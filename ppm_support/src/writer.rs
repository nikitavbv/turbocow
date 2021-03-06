use core::models::{ImageWriter, Image};
pub struct PPMWriter {
}

impl ImageWriter for PPMWriter {
    
    fn write(&self, _image: &Image) -> Vec<u8> {
        println!("writing bmp image");

        Vec::new()
    }
}