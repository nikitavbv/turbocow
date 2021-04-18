use serde::{Serialize, Deserialize};
use turbocow_core::models::pixel::Pixel;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Multi(Vec<Message>),
    SetPixel { x: u16, y: u16, pixel: Pixel },
}