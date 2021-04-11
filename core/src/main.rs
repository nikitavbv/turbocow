#![feature(box_syntax)]

#[macro_use] 
extern crate log;
extern crate libloading;
extern crate custom_error;

pub mod geometry;
pub mod models;
pub mod objects;
pub mod render;
pub mod scene;
pub mod plugins;
pub mod utils;

use std::path::Path;
use std::fs;

use env_logger::Env;
use geometry::{transform::Transform, vector3::Vector3};
use models::image::Image;
use objects::sphere::Sphere;
use plugins::resolver::PluginResolver;
use models::io::ImageWriterOptions;
use render::basic::BasicRender;
use scene::{scene::Scene, camera::Camera};

const DEFAULT_LOGGING_LEVEL: &str = "info";

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or(DEFAULT_LOGGING_LEVEL)).init();
    utils::print_intro();

    let mut plugin_resolver = PluginResolver::new(box Path::new("plugins"))
        .expect("Failed to init plugin resolver");
    render_test_scene(&mut plugin_resolver);

    info!("done");
}

fn render_test_scene(plugin_resolver: &mut PluginResolver) {
    let bmp_support = plugin_resolver.resolve_or_install_image_support("bmp");

    let mut scene = Scene::new();
    scene.set_camera(Camera::default());
    scene.add_object(box Sphere::new(Transform::new(&Vector3::new(0.0, 0.0, -5.0)), 1.0));

    let render = BasicRender::new();
    let mut output = Image::new(1000, 1000);

    info!("rendering image");
    render.render(&scene, &mut output);

    info!("saving rendered image");
    fs::write("result.bmp", &bmp_support.writer().write(&output, &ImageWriterOptions::default()).expect("Failed to write image as bmp"))
        .expect("failed to save result image")
}