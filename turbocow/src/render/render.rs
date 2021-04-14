use turbocow_core::models::image::Image;

use crate::scene::scene::Scene;

pub trait Render {

    fn render(&self, scene: &Scene, render_to: &mut Image);
}