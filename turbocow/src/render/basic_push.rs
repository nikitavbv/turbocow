use core::f64::consts::PI;

use turbocow_core::models::pixel::Pixel;
use turbocow_core::models::image::Image;
use livestonk::Component;

use crate::{geometry::{ray::Ray, vector3::Vector3}, scene::{scene::Scene, scene_object::SceneObject}};

use super::render::Render;
use crate::render::intersection::Intersection;
use std::net::{UdpSocket, SocketAddrV4, Ipv4Addr, TcpStream};
use std::time::Instant;
use std::io::Write;

use byteorder::{LittleEndian, WriteBytesExt};
use std::convert::TryInto;
use crate::protocol::message::Message;
use crate::protocol::socket::CowSocket;
use crate::render::render::RenderError;
use crate::render::basic::render_ray;

#[derive(Component)]
pub struct BasicPushRender {
}

impl BasicPushRender {

    pub fn new() -> Self {
        Self {
        }
    }
}

impl Render for BasicPushRender {

    fn render(&self, scene: &Scene, render_to: &mut Image) -> Result<(), RenderError> {
        let socket = CowSocket::start_client(Ipv4Addr::LOCALHOST)?;

        let camera = scene.camera();

        let transform = camera.transform();
        let width = render_to.width;
        let height = render_to.height;
        let aspect_ratio = width as f64 / height as f64;
        let field_of_view = (camera.field_of_view() / 2.0).tan();

        let transformed_origin = transform.apply_for_point(transform.position());

        for y in 0..height {
            let normalized_y = 1.0 - 2.0 * (y as f64 + 0.5) / height as f64;
            let camera_y = normalized_y * field_of_view;

            for x in 0..width {
                let normalized_x = 2.0 * (x as f64 + 0.5) / (width as f64) - 1.0;
                let camera_x = normalized_x * aspect_ratio * field_of_view;

                let direction = Vector3::new(camera_x, camera_y, -1.0).normalized();

                let ray = Ray::new(
                    transformed_origin,
                    transform.apply_for_vector(&direction).normalized()
                );
                let pixel = render_ray(&ray, &scene);

                socket.send(Message::SetPixel {
                    x: x as u16,
                    y: y as u16,
                    pixel,
                }, true);
            }
        }

        Ok(())
    }

    fn is_remote_write(&self) -> bool {
        true
    }
}