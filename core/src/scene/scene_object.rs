use crate::geometry::ray::Ray;

pub trait SceneObject {

    fn intersects(&self, ray: &Ray) -> bool;
}