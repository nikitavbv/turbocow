use crate::geometry::transform::Transform;
use crate::geometry::vector3::Vector3;

pub trait Light {

    fn transform(&self) -> &Transform;

    fn illuminate(&self, normal: &Vector3, distance: f64) -> f64;

    fn max_distance(&self, point: &Vector3) -> f64 {
        point.distance_to(&self.transform().position())
    }
}