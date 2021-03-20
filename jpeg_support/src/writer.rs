use core::models::io::ImageWriter;

pub struct JPEGWriter {
}

impl ImageWriter for JPEGWriter {
    
    fn write(&self, image: &core::models::image::Image, options: &core::models::io::ImageWriterOptions) -> Result<Vec<u8>, core::models::io::ImageIOError> {
        todo!()
    }
}