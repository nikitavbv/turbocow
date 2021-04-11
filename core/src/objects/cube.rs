use crate::{geometry::{ray::Ray, transform::Transform}, render::intersection::Intersection, scene::scene_object::SceneObject, geometry::vector3::Vector3};

const DELTA: f64 = 1e-6;

pub struct Cube {
    upper_bounds: Vector3,
    lower_bounds: Vector3,
}

impl Cube {

    pub fn new(transform: Transform, size: f64) -> Self {
        let upper_bounds = transform.position() + &(Vector3::one() * size * 0.5);
        let lower_bounds = transform.position() - &(Vector3::one() * size * 0.5);

        Self {
            upper_bounds,
            lower_bounds,
        }
    }
}

impl SceneObject for Cube {

    fn check_intersection(&self, ray: &Ray) -> Option<Intersection> {
        let direction = ray.direction();
        let origin = ray.origin();
        let x = axis_direction(direction.x);
        let y = axis_direction(direction.y);
        let z = axis_direction(direction.z);

        let (t_min, t_max) = if direction.x >= 0.0 {
            (
                (self.lower_bounds.x - origin.x) * x,
                (self.upper_bounds.x - origin.x) * x,
            )
        } else {
            (
                (self.upper_bounds.x - origin.x) * x,
                (self.lower_bounds.x - origin.x) * x,
            )
        };

        let (ty_min, ty_max) = if direction.y >= 0.0 {
            (
                (self.lower_bounds.y - origin.y) * y,
                (self.upper_bounds.y - origin.y) * y,
            )
        } else {
            (
                (self.upper_bounds.y - origin.y) * y,
                (self.lower_bounds.y - origin.y) * y,
            )
        };

        if t_min > ty_max || ty_min > t_max {
            return None;
        }
        
        let t_min = ty_min.max(t_min);
        let t_max = ty_max.min(t_max);

        let (tz_min, tz_max) = if direction.z >= 0.0 {
            (
                (self.lower_bounds.z - origin.z) * z,
                (self.upper_bounds.z - origin.z) * z,
            )
        } else {
            (
                (self.upper_bounds.z - origin.z) * z,
                (self.lower_bounds.z - origin.z) * z,
            )
        };

        if t_min > tz_max || tz_min > t_max {
            return None;
        }

        let t_min = tz_min.max(t_min);
        let t_max = tz_max.min(t_max);

        let t = if t_min < 0.0 {
            if t_max < 0.0 {
                return None
            }
            t_max
        } else if t_min > t_max {
            t_max
        } else {
            t_min
        };

        Some(Intersection::new(t))
    }
}

fn axis_direction(val: f64) -> f64 {
    if val.abs() < DELTA {
        if val >= 0.0 {
            1.0 / DELTA
        } else {
            -1.0 / DELTA
        }
    } else {
        1.0 / val
    }
}