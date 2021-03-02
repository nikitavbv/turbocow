use core::models::{ImageReader, Image};

pub struct BMPReader {
}

impl ImageReader for BMPReader {


    fn read(&self, _data: &[u8]) -> Image {
        println!("reading bmp image");

        Image {
            width: 0,
            height: 0,
            pixels: Vec::new(),
        }
    }
}