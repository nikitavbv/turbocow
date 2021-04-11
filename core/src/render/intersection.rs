pub struct Intersection {

    ray_distance: f64,
}

impl Intersection {

    pub fn new(ray_distance: f64) -> Self {
        Self {
            ray_distance
        }
    }

    pub fn ray_distance(&self) -> f64 {
        self.ray_distance
    }
}