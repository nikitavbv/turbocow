use crate::{render::intersection::Intersection, geometry::{ray::Ray, transform::Transform, vector3::Vector3}, scene::scene_object::SceneObject};

const DELTA: f64 = 1e-6;

#[derive(Clone)]
pub struct Triangle {

    transform: Transform,

    v0: Vector3,
    v1: Vector3,
    v2: Vector3,

    v0_applied: Vector3,

    v0v1: Vector3,
    v0v2: Vector3,

    // normals
    vn0: Vector3,
    vn1: Vector3,
    vn2: Vector3,
}

impl Triangle {

    pub fn new(transform: Transform, v0: Vector3, v1: Vector3, v2: Vector3) -> Self {
        let v0_applied = transform.apply_for_point(&v0);
        let v1_applied = transform.apply_for_point(&v1);
        let v2_applied = transform.apply_for_point(&v2);

        let v0v1 = v1_applied - v0_applied;
        let v0v2 = v2_applied - v0_applied;

        Self {
            transform,

            v0,
            v1,
            v2,

            v0_applied,

            v0v1,
            v0v2,

            vn0: Vector3::zero(),
            vn1: Vector3::zero(),
            vn2: Vector3::zero(),
        }
    }

    pub fn new_with_normals(transform: Transform, v0: Vector3, v1: Vector3, v2: Vector3, vn0: Vector3, vn1: Vector3, vn2: Vector3) -> Self {
        let v0_applied = transform.apply_for_point(&v0);
        let v1_applied = transform.apply_for_point(&v1);
        let v2_applied = transform.apply_for_point(&v2);

        let v0v1 = v1_applied - v0_applied;
        let v0v2 = v2_applied - v0_applied;

        Self {
            transform,

            v0,
            v1,
            v2,

            v0_applied,

            v0v1,
            v0v2,

            vn0,
            vn1,
            vn2,
        }
    }

    pub fn get_vertices(&self) -> [&Vector3; 3] {
        [&self.v0, &self.v1, &self.v2]
    }
}

impl SceneObject for Triangle {

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn check_intersection(&self, ray: &Ray) -> Option<Intersection> {
        let v0 = &self.v0_applied;
        let v0v1 = &self.v0v1;
        let v0v2 = &self.v0v2;

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

        let w = 1.0 - u - v;
        let na = self.vn0.clone() * w;
        let nb = self.vn1.clone() * u;
        let nc = self.vn2.clone() * v;

        // return Some(Intersection::new(ray_distance, Some(self.v0v1.cross_product(&self.v0v2).normalized())))
        Some(Intersection::new(ray_distance, Some((na + nb + nc).normalized())))
    }
}