#![feature(box_syntax)]

#[macro_use] 
extern crate log;
extern crate libloading;
extern crate custom_error;

pub mod models;
pub mod render;
pub mod scene;
pub mod plugins;
pub mod utils;

use std::path::Path;

use env_logger::Env;
use models::image::Image;
use plugins::PluginManager;
use render::basic::BasicRender;
use scene::scene::Scene;

const DEFAULT_LOGGING_LEVEL: &str = "info";
const PLUGINS_DIR: &str = "plugins";

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or(DEFAULT_LOGGING_LEVEL)).init();
    utils::print_intro();

    let mut plugin_manager = PluginManager::new();
    if let Err(err) = plugin_manager.load_plugins(box Path::new(PLUGINS_DIR)) {
        error!("failed to load plugins: {}", err);
    }

    render_test_scene();

    info!("done");
}

fn render_test_scene() {
    let scene = Scene::new();
    let render = BasicRender::new();
    let mut output = Image::new(100, 100);

    render.render(&scene, &mut output);
}