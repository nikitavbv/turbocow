use crate::geometry::vector3::Vector3;
use turbocow_core::models::pixel::Pixel;

#[derive(Debug, Clone)]
pub enum Material {
    Lambertian { albedo: f64, color: Pixel },
    Reflective,
}

pub fn reflect(i: &Vector3, n: &Vector3) -> Vector3 {
    i.clone() - 2.0 * i.dot_product(n) * n
}

pub fn refract(i: &Vector3, ni: &Vector3, ior: f64) -> Vector3 {
    let mut cosi = i.dot_product(ni).max(-1.0).min(1.0);
    let mut etai = 1.0;
    let mut etat = ior;
    let mut n = ni.clone();

    if cosi < 0.0 {
        cosi = -cosi;
    } else {
        let t = etai;
        etai = etat;
        etat = t;
        n = -1.0 * ni;
    }

    let eta = etai / etat;
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);

    (if k < 0.0 {
        0.0
    } else {
        eta
    }) * i + (eta * cosi - k.sqrt()) * n
}