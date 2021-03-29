use std::collections::HashMap;

use core::models::pixel::Pixel;

#[derive(Clone)]
pub struct Channel {
    pub id: u8,
    pub horizontal_sampling: u8,
    pub vertical_sampling: u8,
    pub quantization_table_id: u8,
}

#[derive(Clone)]
pub struct HuffmanTable {

    pub id: u8,
    pub table_type: HuffmanTableType,
    pub table: HashMap<(u16, u16), u8>,
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum HuffmanTableType {
    DC,
    AC,
}

pub fn rgb_to_ycbcr(pixel: &Pixel) -> (i32, i32, i32) {
    let red = pixel.red as f32;
    let green = pixel.green as f32;
    let blue = pixel.blue as f32;
    (
        (0.299 * red + 0.587 * green + 0.114 * blue).round() as i32,
        (128.0 - 0.168736 * red - 0.331264 * green + 0.5 * blue).round() as i32,
        (128.0 + 0.5 * red - 0.418688 * green - 0.081312 * blue).round() as i32,
    )
}

pub fn ycbcr_to_rgb(y: i32, cb: i32, cr: i32) -> (u8, u8, u8) {
    let r = ((y as f32 + 1.402 * (cr as f32 - 128.0)).round() as i32).max(0).min(255) as u8;
    let g = ((y as f32 - 0.34414 * (cb as f32 - 128.0) - 0.71414 * (cr as f32 - 128.0)).round() as i32).max(0).min(255) as u8;
    let b = ((y as f32 + 1.772 * (cb as f32 - 128.0)).round() as i32).max(0).min(255) as u8;
    (r, g, b)
}