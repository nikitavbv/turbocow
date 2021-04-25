#![feature(box_syntax)]
#![feature(control_flow_enum)]

#[macro_use] 
extern crate log;
extern crate custom_error;

pub mod ui;
pub mod geometry;
pub mod io;
pub mod objects;
pub mod protocol;
pub mod render;
pub mod scene;
pub mod scenes;

use std::path::Path;
use std::{fs, env, thread};

use env_logger::Env;

use turbocow_core::utils::print_intro;
use turbocow_core::models::image::Image;
use turbocow_core::plugins::plugins::ImageFormatSupportPlugin;
use turbocow_core::models::io::ImageWriterOptions;
use livestonk::{bind, Livestonk};
use bmp_support::BMPFormatSupportPlugin;

use geometry::{ray::Ray, transform::Transform, vector3::Vector3};
use io::obj::obj_file_reader::ObjFileLoader;
use objects::{polygon_object::PolygonObject, triangle::Triangle};
use render::{multithreaded::MultithreadedRender, render::Render};
use scene::{camera::Camera, scene::Scene, scene_object::SceneObject};
use crate::render::basic::BasicRender;
use crate::render::basic_push::BasicPushRender;
use crate::io::traits::{Model, ModelLoader};
use crate::scenes::provider::SceneProvider;
use crate::scenes::demo::DemoSceneProvider;
use std::collections::{HashSet, HashMap};
use crate::ui::window::WindowOutput;

const DEFAULT_LOGGING_LEVEL: &str = "info";

livestonk::init!();

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or(DEFAULT_LOGGING_LEVEL)).init();
    print_intro();

    //livestonk::bind!(dyn Render, MultithreadedRender);
    livestonk::bind!(dyn Render, BasicPushRender);

    livestonk::bind_to_instance!(dyn ImageFormatSupportPlugin, BMPFormatSupportPlugin::new());
    livestonk::bind!(dyn ModelLoader, ObjFileLoader);
    livestonk::bind!(dyn SceneProvider, DemoSceneProvider);

    run();

    info!("done");
}

fn run() {
    let args: Vec<String> = env::args().collect();
    let flags: HashSet<String> = args.iter()
        .filter(|arg| arg.starts_with("--"))
        .map(|flag| &flag[2..])
        .map(|v| v.to_string())
        .collect();

    let options: HashMap<String, String> = flags.iter()
        .filter(|v| v.contains("="))
        .map(|v| (v[0..v.find("=").unwrap()].to_string(), v[v.find("=").unwrap()+1..].to_string()))
        .collect();

    let commands: Vec<String> = args.iter()
        .filter(|v| !v.starts_with("--"))
        .map(|v| v.clone())
        .collect();

    if commands.len() == 1 {
        render_scene(flags, options);
        return;
    }

    match commands[1].as_str() {
        "render" => render_scene(flags, options),
        "ui" => ui::window::run_with_args(&commands[2..]),
        other => error!("Unknown mode: {}", other)
    }
}

fn render_scene(flags: HashSet<String>, options: HashMap<String, String>) {
    let display_join_handle = if flags.contains("display") {
        Some(thread::spawn(|| WindowOutput::new().update_loop()))
    } else {
        None
    };

    let output_format_support: Box<dyn ImageFormatSupportPlugin> = Livestonk::resolve();
    let scene_provider: Box<dyn SceneProvider> = Livestonk::resolve();
    let render: Box<dyn Render> = Livestonk::resolve();

    let scene = scene_provider.scene(&options);
    let mut output = Image::new(1000, 1000);

    info!("rendering image");
    render.render(&scene, &mut output);

    if let Some(handle) = display_join_handle {
        info!("done rendering image");
        handle.join().expect("Failed to join display thread");
    } else {
        info!("saving rendered image");
        let image_bytes = output_format_support.writer()
            .write(&output, &ImageWriterOptions::default())
            .expect("failed to write image");
        fs::write(options.get("output").unwrap_or(&"result.bmp".to_string()), &image_bytes)
            .expect("failed to save result image");
    }
}
