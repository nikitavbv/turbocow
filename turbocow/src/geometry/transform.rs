use super::{matrix::Matrix44, vector3::Vector3};

#[derive(Clone)]
pub struct Transform {

    position: Vector3,

    matrix: Matrix44,
}

impl Transform {

    pub fn new(position: &Vector3) -> Self {
        Self {
            position: position.clone(),
            matrix: Matrix44::for_transformation(position),
        }
    }

    pub fn default() -> Self {
        Self::new(&Vector3::zero())
    }

    pub fn position(&self) -> &Vector3 {
        &self.position
    }

    pub fn apply_for_point(&self, point: &Vector3) -> Vector3 {
        self.matrix.apply_for_point(point)
    }

    pub fn apply_for_vector(&self, vector: &Vector3) -> Vector3 {
        self.matrix.apply_for_vector(vector)
    }
}