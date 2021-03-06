use crate::{geometry::{ray::Ray, transform::Transform}, render::intersection::Intersection};
use crate::scene::scene_object::SceneObject;
use crate::geometry::vector3::Vector3;
use crate::materials::material::Material;

pub struct Sphere {

    id: usize,
    transform: Transform,
    material: Material,
    radius: f64,
}

impl Sphere {

    pub fn new(id: usize, transform: Transform, material: Material, radius: f64) -> Self {
        Self {
            id,
            transform,
            material,
            radius,
        }
    }
}

impl SceneObject for Sphere {

    fn id(&self) -> usize {
        self.id
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn material(&self) -> Material {
        self.material.clone()
    }

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

        let first_intersection = (-b + discriminant.sqrt()) / (2.0 * a);
        let second_intersection = (-b - discriminant.sqrt()) / (2.0 * a);

        if first_intersection < 0.0 && second_intersection < 0.0 {
            return None;
        }

        let t = if first_intersection < 0.0 && second_intersection >= 0.0 {
            second_intersection
        } else if first_intersection >= 0.0 && first_intersection < 0.0 {
            first_intersection
        } else {
            first_intersection.min(second_intersection)
        };

        let hit_point = ray.origin() + &(ray.direction() * t);
        return Some(Intersection::new(t, Some((&hit_point - self.transform.position()).normalized())));
    }
}