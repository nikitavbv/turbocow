use serde::{Serialize, Deserialize};
use turbocow_core::models::pixel::Pixel;
use std::convert::TryInto;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Ping,
    Pong,
    Flush,
    Close,
    StartStreaming,
    Batch (Box<[Message; 32]>),
    BatchLarge (Vec<Message>),
    SetPixel { x: u16, y: u16, pixel: Pixel },
}
