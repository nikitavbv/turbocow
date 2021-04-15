use crate::scene::light::Light;
use crate::geometry::vector3::Vector3;
use crate::geometry::transform::Transform;

pub struct DistantLight {

    transform: Transform,
    intensity: f64,
}

impl DistantLight {

    pub fn new(transform: Transform, intensity: f64) -> Self {
        DistantLight {
            transform,
            intensity
        }
    }
}

impl Light for DistantLight {

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn illuminate(&self, normal: &Vector3, _distance: f64) -> f64 {
        (self.intensity * normal.dot_product(&(self.transform.rotation() * -1.0))).max(0.0)
    }

    fn max_distance(&self, _point: &Vector3) -> f64 {
        f64::MAX
    }
}