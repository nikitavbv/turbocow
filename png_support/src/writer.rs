use core::models::{ImageWriter, Image, ImageIOError};
pub struct PNGWriter {
}

impl PNGWriter {
    const fn new() -> Self {
        PNGWriter {}
    }
}

impl ImageWriter for PNGWriter {
    
    fn write(&self, image: &Image) -> Result<Vec<u8>, ImageIOError> {
        Result::Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple_test() {
    }
}