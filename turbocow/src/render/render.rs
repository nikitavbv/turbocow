use custom_error::custom_error;

use turbocow_core::models::image::Image;

use crate::scene::scene::Scene;
use crate::protocol::socket::CowSocketError;

custom_error! {pub RenderError
    SocketError {source: CowSocketError} = "Socket error: {source}",
}

pub trait Render {

    fn render(&self, scene: &Scene, render_to: &mut Image) -> Result<(), RenderError>;

    fn is_remote_write(&self) -> bool {
        false
    }
}