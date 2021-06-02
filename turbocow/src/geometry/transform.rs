use super::{matrix::Matrix44, vector3::Vector3};

#[derive(Clone)]
pub struct Transform {

    position: Vector3,
    rotation: Vector3,

    matrix: Matrix44,
}

impl Transform {

    pub fn new(position: Vector3, rotation: Vector3) -> Self {
        let matrix = Matrix44::for_transformation(&position, &rotation);

        Self {
            position,
            rotation,
            matrix,
        }
    }

    pub fn default() -> Self {
        Self::new(Vector3::zero(), Vector3::zero())
    }

    pub fn from_scene_format(transform: &sceneformat::Transform) -> Self {
        let position = match &transform.position {
            Some(v) => Vector3::new(v.x, v.y, v.z),
            None => Vector3::zero(),
        };
        let rotation = match &transform.rotation {
            Some(v) => Vector3::new(v.x, v.y, v.z),
            None => Vector3::zero(),
        };

        Transform::new(position, rotation)
    }

    pub fn position(&self) -> &Vector3 {
        &self.position
    }

    pub fn rotation(&self) -> &Vector3 {
        &self.rotation
    }

    pub fn apply_for_point(&self, point: &Vector3) -> Vector3 {
        self.matrix.apply_for_point(point)
    }

    pub fn apply_for_vector(&self, vector: &Vector3) -> Vector3 {
        self.matrix.apply_for_vector(vector)
    }
}
