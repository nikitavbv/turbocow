use crate::{render::intersection::Intersection, geometry::{ray::Ray, transform::Transform}};
use crate::geometry::vector3::Vector3;
use turbocow_core::models::pixel::Pixel;
use crate::materials::material::Material;

pub trait SceneObject {

    fn transform(&self) -> &Transform;

    fn check_intersection(&self, ray: &Ray) -> Option<Intersection>;

    fn material(&self) -> Material {
        Material::Lambertian {
            albedo: 0.18,
            color: Pixel::from_rgb(20, 20, 220),
        }
    }
}