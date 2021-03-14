#![feature(box_syntax)]
#![feature(destructuring_assignment)]

#[macro_use] 
extern crate log;
extern crate custom_error;

use core::{plugins::ImageFormatSupportPlugin, models::{ImageReader, ImageWriter}};

use reader::GIFReader;
use writer::GIFWriter;

mod clustering;
pub mod reader;
pub mod writer;

pub struct GIFFormatSupportPlugin {
}

impl GIFFormatSupportPlugin {

    pub fn new() -> Self {
        GIFFormatSupportPlugin {}
    }
}

impl ImageFormatSupportPlugin for GIFFormatSupportPlugin {

    fn format_name(&self) -> String {
        "GIF".to_string()
    }

    fn reader(&self) -> Box<dyn ImageReader> {
        box GIFReader {}
    }

    fn writer(&self) -> Box<dyn ImageWriter> {
        box GIFWriter {}
    }
}

#[no_mangle]
pub fn _plugin_init() -> Box<dyn ImageFormatSupportPlugin> {
    box GIFFormatSupportPlugin::new()
}
