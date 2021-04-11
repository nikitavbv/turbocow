use super::{matrix::Matrix44, vector3::Vector3};

pub struct Transform {

    position: Vector3,
}

impl Transform {

    pub fn position(&self) -> &Vector3 {
        &self.position
    }
}