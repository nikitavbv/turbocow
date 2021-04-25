use rayon::prelude::*;

use turbocow_core::models::pixel::Pixel;
use turbocow_core::models::image::Image;
use livestonk::*;

use crate::{geometry::{ray::Ray, vector3::Vector3}, scene::{scene::Scene, scene_object::SceneObject}};

use super::render::Render;
use crate::render::basic::render_ray;
use crate::render::render::RenderError;

#[derive(Component)]
pub struct MultithreadedRender {
}

impl MultithreadedRender {

    pub fn new() -> Self {
        Self {
        }
    }
}

impl Render for MultithreadedRender {

    fn render(&self, scene: &Scene, render_to: &mut Image) -> Result<(), RenderError> {
        let width = render_to.width;
        let height = render_to.height;
        let chunk_size = width;

        render_to.pixels.par_chunks_mut(chunk_size).enumerate().for_each(|(i, output)| {
            worker(&scene, output, i, chunk_size, width, height);
        });

        Ok(())
    }
}

fn worker(scene: &Scene, output: &mut [Pixel], chunk: usize, chunk_size: usize, width: usize, height: usize) {
    let camera = scene.camera();

    let transform = camera.transform();
    let aspect_ratio = width as f64 / height as f64;
    let field_of_view = (camera.field_of_view() / 2.0).tan();

    let transformed_origin = transform.apply_for_point(transform.position());

    let y = chunk;
    let normalized_y = 1.0 - 2.0 * (y as f64 + 0.5) / height as f64;
    let camera_y = normalized_y * field_of_view;

    for x in 0..chunk_size {
        let normalized_x = 2.0 * (x as f64 + 0.5) / (1000 as f64) - 1.0;
        let camera_x = normalized_x * aspect_ratio * field_of_view;

        let direction = Vector3::new(camera_x, camera_y, -1.0).normalized();
        let ray = Ray::new(transformed_origin, transform.apply_for_vector(&direction).normalized());

        output[x] = render_ray(&ray, &scene);
    }
}
