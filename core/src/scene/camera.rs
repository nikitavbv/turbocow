use crate::{render::intersection::Intersection, geometry::ray::Ray};

use super::scene_object::{SceneObject, SceneObjectBase};

pub struct Camera {
    base: SceneObjectBase,

    field_of_view: f64,
}

impl Camera {

    pub fn new(size: f64) -> Self {
        Self {
            base: SceneObjectBase::new(),
            field_of_view: 2.0 * size.atan(),
        }
    }

    pub fn default() -> Self {
        Self::new(1.0)
    }

    pub fn field_of_view(&self) -> f64 {
        self.field_of_view
    }
}

impl SceneObject for Camera {

fn check_intersection(&self, _ray: &Ray) -> Option<Intersection> {
        panic!("Why are you even trying to find an intersection with a camera?")
    }

    fn base(&self) -> &SceneObjectBase {
        &self.base
    }
}