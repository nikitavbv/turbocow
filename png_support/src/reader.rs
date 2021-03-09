use core::models::{ImageReader, Image, ImageIOError};

pub struct PNGReader {
}

impl PNGReader {
    const fn new() -> Self {
        PNGReader {}
    }
}

impl ImageReader for PNGReader {

    fn read(&self, data: &Vec<u8>) -> Result<Vec<Image>, ImageIOError> {
        Result::Ok(Vec::new())
    }

}

#[cfg(test)]
mod tests {
    #[test]
    fn test_simple() {
    }
}