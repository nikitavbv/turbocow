use crate::{geometry::{ray::Ray, transform::Transform}, render::intersection::Intersection};
use crate::scene::scene_object::{SceneObject, SceneObjectBase};

pub struct Sphere {

    transform: Transform,
    radius: f64,
}

impl Sphere {

    pub fn new(transform: Transform, radius: f64) -> Self {
        Self {
            transform,
            radius,
        }
    }
}

impl SceneObject for Sphere {
    
    fn check_intersection(&self, ray: &Ray) -> Option<Intersection> {
        // Ray(t) = origin + t * direction
        // Spehere: (x - center_x)**2 + (y - center_y)**2 + (z - center_z)**2 = radius**2
        // Sphere vector form: |Point - Center|**2 = radius**2
        
        // Combined equation is: |origin + t * direction - center|**2 = radius**2
        // Expanded as: t**2 * |direction|**2 + 2t * dot(direction, origin - center) + |origin - center| ** 2 - radius**2 = 0

        // quadratic equation for t:
        // a = |direction|**2
        // b = 2 * dot(direction, origin - center)
        // c = |origin - center| ** 2 - radius**2
        // Discriminant is: b ** 2 - 4 * a * c = (2 * dot(direction, origin - center)) ** 2 - 4 * |direction|**2 * (|origin - center| ** 2 - radius**2)

        // If Dicriminant is < 0, then no intersection, if Discriminant is 0 then one intersection, two otherwise.

        let a = ray.direction().dot_product_with_self();
        let b = 2.0 * ray.direction().dot_product(&(ray.origin() - self.transform.position()));
        let c = (ray.origin() - self.transform.position()).dot_product_with_self() - &self.radius.powi(2);

        let discriminant = b.powi(2) - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        Some(Intersection::new(0.0))
    }
}