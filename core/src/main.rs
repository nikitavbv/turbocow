#![feature(box_syntax)]

#[macro_use] 
extern crate log;
extern crate libloading;
extern crate custom_error;

use std::path::Path;

use env_logger::Env;
use plugins::PluginManager;

pub mod models;
pub mod plugins;
pub mod utils;

const DEFAULT_LOGGING_LEVEL: &str = "info";
const PLUGINS_DIR: &str = "plugins";

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or(DEFAULT_LOGGING_LEVEL)).init();
    utils::print_intro();

    let mut plugin_manager = PluginManager::new();
    if let Err(err) = plugin_manager.load_plugins(box Path::new(PLUGINS_DIR)) {
        error!("failed to load plugins: {}", err);
    }

    info!("done");
}
