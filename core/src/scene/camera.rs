use crate::{geometry::transform::Transform, geometry::vector3::Vector3};

pub struct Camera {

    transform: Transform,
    field_of_view: f64,
}

impl Camera {

    pub fn new(position: Vector3, size: f64) -> Self {
        Self {
            transform: Transform::new(&position),
            field_of_view: 2.0 * size.atan(),
        }
    }

    pub fn default() -> Self {
        Self::new(Vector3::zero(), 1.0)
    }

    pub fn with_transform(&self, transform: Transform) -> Self {
        Self {
            transform,
            field_of_view: self.field_of_view,
        }
    }

    pub fn field_of_view(&self) -> f64 {
        self.field_of_view
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }
}
