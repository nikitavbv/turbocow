#![feature(box_syntax)]
#![feature(destructuring_assignment)]

use core::{models::io::ImageReader, plugins::ImageFormatSupportPlugin, models::io::ImageWriter};

use reader::JPEGReader;
use writer::JPEGWriter;

#[macro_use]
extern crate log;
extern crate custom_error;
extern crate lazy_static;
extern crate maplit;

mod common;
pub mod errors;
mod huffman;
pub mod reader;
pub mod writer;

pub struct JPEGSupportPlugin {
}

impl JPEGSupportPlugin {

    pub fn new() -> Self {
        JPEGSupportPlugin {}
    }
}

impl ImageFormatSupportPlugin for JPEGSupportPlugin {

    fn format_name(&self) -> String {
        "JPG".to_string()
    }

    fn reader(&self) -> Box<dyn ImageReader> {
        box JPEGReader {}
    }

    fn writer(&self) -> Box<dyn ImageWriter> {
        box JPEGWriter {}
    }
}

#[no_mangle]
pub fn _plugin_init() -> Box<dyn ImageFormatSupportPlugin> {
    box JPEGSupportPlugin::new()
}