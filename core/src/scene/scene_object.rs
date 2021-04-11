use crate::{render::intersection::Intersection, geometry::{ray::Ray, transform::Transform}};

pub struct SceneObjectBase {

    transform: Transform,
}

impl SceneObjectBase {

    pub fn new() -> Self {
        SceneObjectBase {
            transform: Transform::default(),
        }
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn with_transform(&self, transform: Transform) -> Self {
        Self {
            transform,
        }
    }
}

pub trait SceneObject {

    fn check_intersection(&self, ray: &Ray) -> Option<Intersection>;

    fn base(&self) -> &SceneObjectBase;
}