use core::f64::consts::PI;

use turbocow_core::models::pixel::Pixel;
use turbocow_core::models::image::Image;
use livestonk::Component;

use crate::{geometry::{ray::Ray, vector3::Vector3}, scene::{scene::Scene, scene_object::SceneObject}};

use super::render::Render;
use crate::render::intersection::Intersection;
use crate::render::render::RenderError;
use crate::materials::material::{Material, reflect};

#[derive(Component)]
pub struct BasicRender {
}

impl BasicRender {

    pub fn new() -> Self {
        Self {
        }
    }
}

impl Render for BasicRender {

    fn render(&self, scene: &Scene, render_to: &mut Image) -> Result<(), RenderError> {
        let camera = scene.camera();

        let transform = camera.transform();
        let width = render_to.width;
        let height = render_to.height;
        let aspect_ratio = width as f64 / height as f64;
        let field_of_view = (camera.field_of_view() / 2.0).tan();

        let transformed_origin = transform.apply_for_point(transform.position());

        for y in 0..height {
            let normalized_y = 1.0 - 2.0 * (y as f64 + 0.5) / height as f64;
            let camera_y = normalized_y * field_of_view;

            for x in 0..width {
                let normalized_x = 2.0 * (x as f64 + 0.5) / (width as f64) - 1.0;
                let camera_x = normalized_x * aspect_ratio * field_of_view;

                let direction = Vector3::new(camera_x, camera_y, -1.0).normalized();

                let ray = Ray::new(
                    transformed_origin,
                    transform.apply_for_vector(&direction).normalized()
                );
                render_to.set_pixel(x, y, render_ray(&ray, &scene));
            }
        }

        Ok(())
    }
}

pub fn render_ray(ray: &Ray, scene: &Scene) -> Pixel {
    render_ray_with_depth(ray, scene, 0)
}

pub fn render_ray_with_depth(ray: &Ray, scene: &Scene, depth: u8) -> Pixel {
    let background = Pixel::from_rgb(192, 212, 250);

    if depth > 10 {
        return background;
    }

    let intersect_obj = find_intersection(&ray, &scene);

    if intersect_obj.is_none() {
        return background;
    }
    let (intersect_obj, intersection) = intersect_obj.unwrap();

    if scene.lights().len() == 0 {
        return match intersect_obj.material() {
            Material::Lambertian { albedo: _, color } => color.clone(),
            _ => panic!("Please add lights to use materials other then lambertian"),
        }
    }

    let hit_point = ray.point(intersection.ray_distance());
    let hit_normal = intersection.normal();

    match intersect_obj.material() {
        Material::Lambertian { albedo, color } => {
            let mut intensity = 0.0;

            for light in scene.lights() {
                let bias = 0.001;

                let ray_to_light = Ray::new(hit_point + hit_normal * bias, light.transform().rotation() * -1.0);

                if find_intersection(&ray_to_light, scene).is_none() {
                    intensity += albedo / PI * light.illuminate(
                        &hit_normal,
                        light.transform().position().distance_to(intersect_obj.transform().position())
                    );
                }
            }

            color.clone() * intensity.min(1.0)
        },
        Material::Reflective => {
            let r = reflect(ray.direction(), hit_normal);
            let bias = 0.001;
            render_ray_with_depth(&Ray::new(hit_point + hit_normal * bias, r), scene, depth + 1) * 0.8
        },
        other => panic!("Material is not implemented: {:?}", other),
    }
}

fn find_intersection<'a>(ray: &Ray, scene: &'a Scene) -> Option<(&'a Box<dyn SceneObject + Sync + Send>, Intersection)> {
    let mut result = None;
    let mut min_distance = f64::MAX;
    let mut result_intersection = None;

    for object in scene.objects() {
        if let Some(intersection) = object.check_intersection(&ray) {
            if intersection.ray_distance() < min_distance {
                min_distance = intersection.ray_distance();
                result = Some(object);
                result_intersection = Some(intersection);
            }
        }
    }

    result.map(|v| (v, result_intersection.unwrap()))
}