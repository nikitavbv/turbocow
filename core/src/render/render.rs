use crate::{models::image::Image, scene::scene::Scene};

pub trait Render {

    fn render(&self, scene: &Scene, render_to: &mut Image);
}