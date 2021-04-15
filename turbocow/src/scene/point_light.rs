use std::f64::consts::PI;

use crate::geometry::transform::Transform;
use crate::geometry::vector3::Vector3;
use crate::scene::light::Light;

pub struct PointLight {

    transform: Transform,
    intensity: f64,
}

impl PointLight {

    pub fn new(transform: Transform, intensity: f64) -> Self {
        PointLight {
            transform,
            intensity
        }
    }
}

impl Light for PointLight {

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn illuminate(&self, normal: &Vector3, distance: f64) -> f64 {
        (self.intensity * normal.dot_product(&(self.transform.rotation() * -1.0)) / (4.0 * PI * distance.powi(2))).max(0.0)
    }
}