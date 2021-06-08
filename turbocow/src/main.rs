#![feature(box_syntax)]
#![feature(control_flow_enum)]

#[macro_use] 
extern crate log;
extern crate custom_error;
extern crate redis;

pub mod distributed;
pub mod geometry;
pub mod materials;
pub mod objects;
pub mod protocol;
pub mod render;
pub mod scene;
pub mod scenes;
pub mod ui;

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
use objects::{polygon_object::PolygonObject, triangle::Triangle};
use render::{multithreaded::MultithreadedRender, render::Render};
use scene::{camera::Camera, scene::Scene, scene_object::SceneObject};
use crate::render::basic::BasicRender;
use crate::render::basic_push::BasicPushRender;
use crate::render::multithreaded_push::MultithreadedPushRender;
use crate::scenes::provider::SceneProvider;
use crate::scenes::sceneformat::SceneFormatLoader;
use std::collections::{HashSet, HashMap};
use crate::ui::window::WindowOutput;
use crate::render::render::RenderError;
use crate::render::streaming::run_streaming_render;
use crate::scenes::pack::run_pack;
use crate::distributed::runner::run_distributed;

const DEFAULT_LOGGING_LEVEL: &str = "info";

livestonk::init!();

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or(DEFAULT_LOGGING_LEVEL)).init();
    print_intro();

    livestonk::bind!(dyn Render, MultithreadedPushRender);

    livestonk::bind_to_instance!(dyn ImageFormatSupportPlugin, BMPFormatSupportPlugin::new());
    livestonk::bind!(dyn SceneProvider, SceneFormatLoader);

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
        "connectivity_test" => protocol::connectivity_test::run_with_args(&commands[2..]),
        "streaming_render" => run_streaming_render(),
        "pack" => run_pack(options),
        "distributed" => run_distributed(&commands[2..], &options),
        other => error!("Unknown mode: {}", other)
    }
}

fn render_scene(flags: HashSet<String>, options: HashMap<String, String>) {
    let display_join_handle = if flags.contains("display") {
        Some(thread::spawn(|| WindowOutput::new_server().update_loop()))
    } else {
        None
    };

    let output_format_support: Box<dyn ImageFormatSupportPlugin> = Livestonk::resolve();
    let scene_provider: Box<dyn SceneProvider> = Livestonk::resolve();
    let render: Box<dyn Render> = Livestonk::resolve();

    let mut used_remote_write = render.is_remote_write();

    let scene = scene_provider.scene(&options);
    let mut output = Image::new(1000, 1000);

    info!("rendering image");
    if let Err(err) = render.render(&scene, &mut output) {
        match err {
            RenderError::SocketError { source: _ } => {
                warn!("Failed to connect via cow socket. Falling back to simple multithreaded renderer...");
                let render: Box<MultithreadedRender> = Livestonk::resolve();
                used_remote_write = render.is_remote_write();
                render.render(&scene, &mut output).unwrap();
            }
        }
    }

    if let Some(handle) = display_join_handle {
        info!("done rendering image");
        handle.join().expect("Failed to join display thread");
    } else if used_remote_write {
        info!("done rendering image. saved using remote write");
    } else {
        info!("saving rendered image");
        let image_bytes = output_format_support.writer()
            .write(&output, &ImageWriterOptions::default())
            .expect("failed to write image");
        fs::write(options.get("output").unwrap_or(&"result.bmp".to_string()), &image_bytes)
            .expect("failed to save result image");
    }
}
