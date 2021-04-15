use super::vector3::Vector3;

pub struct Ray {
    origin: Vector3,
    direction: Vector3,
}

impl Ray {

    pub fn new(origin: Vector3, direction: Vector3) -> Self {
        Ray {
            origin,
            direction,
        } 
    }

    pub fn origin(&self) -> &Vector3 {
        &self.origin
    }

    pub fn direction(&self) -> &Vector3 {
        &self.direction
    }

    pub fn point(&self, distance: f64) -> Vector3 {
        self.origin + self.direction * distance
    }
}