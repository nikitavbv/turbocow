#![feature(exact_size_is_empty)]

use core::models::{ImageWriter, Image};
pub struct PPMWriter {
}

impl ImageWriter for PPMWriter {
    
    fn write(&self, image: &Image) -> Vec<u8> {
        let bytes = Vec::new();
        bytes.extend_from_slice(b"P3");
        // 10 - LF
        bytes.push(10);
        bytes.extend_from_slice(image.width.to_string().as_bytes());
        bytes.push(10);
        bytes.extend_from_slice(image.height.to_string().as_bytes());
        bytes.push(10);
        bytes.extend_from_slice(b"255");
        bytes.push(10);
        let pixels = image.pixels.iter();
        loop {
            for _ in 0..image.width {
                if let Some((red, green, blue)) = pixels.next() {
                    bytes.extend_from_slice(red.to_string().as_bytes());
                    bytes.extend_from_slice(b" ");
                    bytes.extend_from_slice(green.to_string().as_bytes());
                    bytes.extend_from_slice(b" ");
                    bytes.extend_from_slice(blue.to_string().as_bytes());
                    bytes.extend_from_slice(b" ");
                } else {
                    return bytes
                }
            }
            bytes.push(10);
        }
    }
}