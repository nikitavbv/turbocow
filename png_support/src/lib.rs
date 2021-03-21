#![feature(box_syntax)]

extern crate custom_error;

use core::{plugins::ImageFormatSupportPlugin, models::io::{ImageReader, ImageWriter}};

use reader::PNGReader;
use writer::PNGWriter;

pub mod reader;
pub mod writer;
pub mod inflate;
pub mod chunk;
pub mod filter;

pub struct PNGFormatSupportPlugin {
}

impl PNGFormatSupportPlugin {

    pub fn new() -> Self {
        PNGFormatSupportPlugin {}
    }
}

impl ImageFormatSupportPlugin for PNGFormatSupportPlugin {

    fn format_name(&self) -> String {
        "PPM".to_string()
    }

    fn reader(&self) -> Box<dyn ImageReader> {
        box PNGReader {}
    }

    fn writer(&self) -> Box<dyn ImageWriter> {
        box PNGWriter {}
    }
}

#[no_mangle]
pub fn _plugin_init() -> Box<dyn ImageFormatSupportPlugin> {
    box PNGFormatSupportPlugin::new()
}
