#![feature(box_syntax)]

extern crate custom_error;

use core::{plugins::ImageFormatSupportPlugin, models::io::{ImageReader, ImageWriter}};

use reader::BMPReader;
use writer::BMPWriter;

mod common;
pub mod reader;
pub mod writer;

pub struct BMPFormatSupportPlugin {
}

impl BMPFormatSupportPlugin {

    pub fn new() -> Self {
        BMPFormatSupportPlugin {}
    }
}

impl ImageFormatSupportPlugin for BMPFormatSupportPlugin {

    fn format_name(&self) -> String {
        "BMP".to_string()
    }

    fn reader(&self) -> Box<dyn ImageReader> {
        box BMPReader {}
    }

    fn writer(&self) -> Box<dyn ImageWriter> {
        box BMPWriter {}
    }
}

#[no_mangle]
pub fn _plugin_init() -> Box<dyn ImageFormatSupportPlugin> {
    box BMPFormatSupportPlugin::new()
}
