use std::ops::Sub;

#[derive(Copy, Clone, Debug)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3 { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn dot_product(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn dot_product_with_self(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}

impl Sub for Vector3 {
    
    type Output = Vector3;

    fn sub(self, rhs: Self) -> Vector3 {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Sub for &Vector3 {
    
    type Output = Vector3;

    fn sub(self, rhs: Self) -> Vector3 {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}