use crate::geometry::vector3::Vector3;

#[derive(Debug)]
pub enum Material {
    Default,
    Reflective,
}

pub fn reflect(i: &Vector3, n: &Vector3) -> Vector3 {
    i.clone() - 2.0 * i.dot_product(n) * n
}