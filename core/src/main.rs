#![feature(box_syntax)]

extern crate libloading;
extern crate custom_error;

use std::path::Path;

use plugins::PluginManager;

pub mod models;
pub mod plugins;
pub mod utils;

fn main() {
    utils::print_intro();

    let mut plugin_manager = PluginManager::new();
    if let Err(err) = plugin_manager.load_plugins(box Path::new("plugins")) {
        println!("failed to load plugins: {}", err);
    }

    println!("done");
}
