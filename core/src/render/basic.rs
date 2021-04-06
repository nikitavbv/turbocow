use crate::scene::scene::Scene;
use crate::models::image::Image;

pub struct BasicRender {
}

impl BasicRender {

    pub fn new() -> Self {
        Self {
        }
    }

    pub fn render(&self, scene: &Scene, render_to: &mut Image) {
        let camera = scene.camera();
    }
}