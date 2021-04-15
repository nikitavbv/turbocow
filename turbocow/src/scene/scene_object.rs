use crate::{render::intersection::Intersection, geometry::{ray::Ray, transform::Transform}};
use crate::geometry::vector3::Vector3;
use turbocow_core::models::pixel::Pixel;

pub trait SceneObject {

    fn transform(&self) -> &Transform;

    fn check_intersection(&self, ray: &Ray) -> Option<Intersection>;

    fn albedo(&self) -> f64 {
        0.18
    }

    fn color(&self) -> Pixel {
        Pixel::from_rgb(20, 20, 220)
    }
}