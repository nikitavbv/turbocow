use core::models::{ImageReader, ImageWriter};

use reader::BMPReader;
use writer::BMPWriter;

pub mod reader;
pub mod writer;

#[no_mangle]
pub fn init_reader() -> Box<dyn ImageReader> {
    Box::new(BMPReader {})
}

#[no_mangle]
pub fn init_writer() -> Box<dyn ImageWriter> {
    Box::new(BMPWriter {})
}