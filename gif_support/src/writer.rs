use core::models::ImageWriter;

pub struct GIFWriter {
}

impl GIFWriter {

    pub fn new() -> Self {
        GIFWriter {
        }
    }
}

impl ImageWriter for GIFWriter {
    
    fn write(&self, image: &core::models::Image) -> Result<Vec<u8>, core::models::ImageIOError> {
        todo!()
    }
}