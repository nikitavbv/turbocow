use crate::geometry::vector3::Vector3;

pub struct Intersection {

    ray_distance: f64,
    normal: Option<Vector3>,
}

impl Intersection {

    pub fn new(ray_distance: f64, normal: Option<Vector3>) -> Self {
        Self {
            ray_distance,
            normal,
        }
    }

    pub fn ray_distance(&self) -> f64 {
        self.ray_distance
    }

    pub fn normal(&self) -> &Vector3 {
        &self.normal.as_ref().unwrap()
    }
}