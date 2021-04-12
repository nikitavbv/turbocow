use crate::{models::pixel::Pixel, geometry::{ray::Ray, vector3::Vector3}, scene::{scene::Scene, scene_object::SceneObject}};
use crate::models::image::Image;

use super::render::Render;

pub struct BasicRender {
}

impl BasicRender {

    pub fn new() -> Self {
        Self {
        }
    }
}

impl Render for BasicRender {

    fn render(&self, scene: &Scene, render_to: &mut Image) {
        let camera = scene.camera();

        let transform = camera.transform();
        let width = render_to.width;
        let height = render_to.height;
        let aspect_ratio = width as f64 / height as f64;
        let field_of_view = (camera.field_of_view() / 2.0).tan();

        let transformed_origin = transform.apply_for_point(transform.position());

        for y in 0..height {
            println!("y is {}", y);
            let normalized_y = 1.0 - 2.0 * (y as f64 + 0.5) / height as f64;
            let camera_y = normalized_y * field_of_view;

            for x in 0..width {
                let normalized_x = 2.0 * (x as f64 + 0.5) / (width as f64) - 1.0;
                let camera_x = normalized_x * aspect_ratio * field_of_view;

                let direction = Vector3::new(camera_x, camera_y, -1.0).normalized();

                let ray = Ray::new(transformed_origin, transform.apply_for_vector(&direction).normalized());

                let intersect_object = find_intersection(&ray, &scene);
                render_to.set_pixel(x, y, if intersect_object.is_some() {
                    Pixel::from_rgb(255, 0, 0)
                } else {
                    Pixel::black()
                });
            }
        }
    }
}

fn find_intersection<'a>(ray: &Ray, scene: &'a Scene) -> Option<&'a Box<dyn SceneObject + Sync + Send>> {
    let mut result = None;
    let mut min_distance = f64::MAX;

    for object in scene.objects() {
        if let Some(intersection) = object.check_intersection(&ray) {
            if intersection.ray_distance() < min_distance {
                min_distance = intersection.ray_distance();
                result = Some(object);
            }
        }
    }

    result
}