#![feature(box_syntax)]

extern crate custom_error;

use core::{plugins::ImageFormatSupportPlugin, models::io::{ImageReader, ImageWriter}};

use reader::PPMReader;
use writer::PPMWriter;

pub mod reader;
pub mod writer;

pub struct PPMFormatSupportPlugin {
}

impl PPMFormatSupportPlugin {

    pub fn new() -> Self {
        PPMFormatSupportPlugin {}
    }
}

impl ImageFormatSupportPlugin for PPMFormatSupportPlugin {

    fn format_name(&self) -> String {
        "PPM".to_string()
    }

    fn reader(&self) -> Box<dyn ImageReader> {
        box PPMReader {}
    }

    fn writer(&self) -> Box<dyn ImageWriter> {
        box PPMWriter {}
    }
}

#[no_mangle]
pub fn _plugin_init() -> Box<dyn ImageFormatSupportPlugin> {
    box PPMFormatSupportPlugin::new()
}
