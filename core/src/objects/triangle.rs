use crate::{render::intersection::Intersection, geometry::{ray::Ray, transform::Transform, vector3::Vector3}, scene::scene_object::SceneObject};

const DELTA: f64 = 1e-6;

pub struct Triangle {

    transform: Transform,
    v0: Vector3,
    v1: Vector3,
    v2: Vector3,
}

impl Triangle {

    pub fn new(transform: Transform, v0: Vector3, v1: Vector3, v2: Vector3) -> Self {
        Self {
            transform,
            v0,
            v1,
            v2,
        }
    }
}

impl SceneObject for Triangle {
    
    fn check_intersection(&self, ray: &Ray) -> Option<Intersection> {
        let v0 = self.transform.apply_for_point(&self.v0);
        let v1 = self.transform.apply_for_point(&self.v1);
        let v2 = self.transform.apply_for_point(&self.v2);

        let v0v1 = v1 - v0;
        let v0v2 = v2 - v1;

        let direction = ray.direction();

        let p = direction.cross_product(&v0v2);
        let det = v0v1.dot_product(&p);
        if det.abs() < DELTA {
            return None;
        }

        let inv_det = 1.0 / det;

        let t = ray.origin() - &v0;
        let u = t.dot_product(&p) * inv_det;

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = t.cross_product(&v0v1);
        let v = direction.dot_product(&q) * inv_det;
        if v < 0.0 || v + u > 1.0 {
            return None;
        }

        let ray_distance = v0v2.dot_product(&q) * inv_det;

        if ray_distance < 0.0 {
            return None;
        }

        return Some(Intersection::new(ray_distance))
    }
}