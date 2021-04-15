#![feature(box_syntax)]

#[macro_use] 
extern crate log;
extern crate custom_error;

pub mod geometry;
pub mod io;
pub mod objects;
pub mod render;
pub mod scene;
pub mod scenes;

use std::path::Path;
use std::fs;

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
use crate::io::traits::{Model, ModelLoader};
use crate::scenes::provider::SceneProvider;
use crate::scenes::demo::DemoSceneProvider;

const DEFAULT_LOGGING_LEVEL: &str = "info";

livestonk::init!();

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or(DEFAULT_LOGGING_LEVEL)).init();
    print_intro();

    livestonk::bind!(dyn Render, BasicRender);
    livestonk::bind_to_instance!(dyn ImageFormatSupportPlugin, BMPFormatSupportPlugin::new());
    livestonk::bind!(dyn ModelLoader, ObjFileLoader);
    livestonk::bind!(dyn SceneProvider, DemoSceneProvider);

    render_scene();

    info!("done");
}

fn render_scene() {
    let output_format_support: Box<dyn ImageFormatSupportPlugin> = Livestonk::resolve();
    let scene_provider: Box<dyn SceneProvider> = Livestonk::resolve();
    let render: Box<dyn Render> = Livestonk::resolve();

    let scene = scene_provider.scene();
    let mut output = Image::new(1000, 1000);

    info!("rendering image");
    render.render(&scene, &mut output);

    info!("saving rendered image");

    let image_bytes = output_format_support.writer()
        .write(&output, &ImageWriterOptions::default())
        .expect("failed to write image");
    fs::write("result.bmp", &image_bytes).expect("failed to save result image");
}
