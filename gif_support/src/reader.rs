use core::models::ImageReader;

pub struct GIFReader {
}

impl GIFReader {

    pub fn new() -> Self {
        GIFReader {}
    }
}

impl ImageReader for GIFReader {
    
    fn read(&self, data: &Vec<u8>) -> Result<core::models::Image, core::models::ImageIOError> {
        todo!()
    }
}