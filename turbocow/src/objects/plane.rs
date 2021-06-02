use crate::geometry::transform::Transform;
use crate::scene::scene_object::SceneObject;
use crate::geometry::ray::Ray;
use crate::render::intersection::Intersection;
use crate::geometry::vector3::Vector3;
use crate::materials::material::Material;

const DELTA: f64 = 1e-10;

pub struct Plane {

    transform: Transform,
    material: Material,
}

impl Plane {

    pub fn new(transform: Transform, material: Material) -> Self {
        Self {
            transform,
            material,
        }
    }
}

impl SceneObject for Plane {

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn material(&self) -> Material {
        self.material.clone()
    }

    fn check_intersection(&self, ray: &Ray) -> Option<Intersection> {
        let origin = self.transform.position();
        let normal = self.transform.apply_for_vector(&(Vector3::up() * -1.0));
        let angle = normal.dot_product(ray.direction());

        if angle <= DELTA {
            return None;
        }

        let direction = origin - ray.origin();
        let scale = direction.dot_product(&normal) / angle;
        if scale < 0.0 {
            return None;
        }

        Some(Intersection::new(scale, Some(normal.normalized() * -1.0)))
    }
}