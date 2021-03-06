use core::models::{ImageReader, Image};

trait RasterReader {

}

pub struct P3RasterReader {
}

impl RasterReader for P3RasterReader {
    
}

pub struct PPMReader {
}

impl ImageReader for PPMReader {


    fn read(&self, _data: &[u8]) -> Image {
        println!("reading bmp image");

        Image {
            width: 0,
            height: 0,
            pixels: Vec::new(),
        }
    }
}