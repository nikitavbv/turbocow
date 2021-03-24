use std::f32::consts::PI;

use core::models::{image::Image, pixel::Pixel, io::{ImageIOError, ImageWriter, ImageWriterOptions}};
use std::{collections::HashMap, convert::TryInto};

const QUANTIZATION_TABLE_Y: [i32; 64] = [
     3,  2,  2,  3,  4,  6,  8, 10, 
     2,  2,  2,  3,  4,  9, 10,  9, 
     2,  2,  3,  4,  6,  9, 11,  9, 
     2,  3,  4,  5,  8, 14, 13, 10, 
     3,  4,  6,  9, 11, 17, 16, 12, 
     4,  6,  9, 10, 13, 17, 18, 15, 
     8, 10, 12, 14, 16, 19, 19, 16, 
    12, 15, 15, 16, 18, 16, 16, 16
];

const QUANTIZATION_TABLE_CB_CR: [i32; 64] = [
     3,  3,  4,  8, 16, 16, 16, 16, 
     3,  3,  4, 11, 16, 16, 16, 16, 
     4,  4,  9, 16, 16, 16, 16, 16, 
     8, 11, 16, 16, 16, 16, 16, 16, 
    16, 16, 16, 16, 16, 16, 16, 16, 
    16, 16, 16, 16, 16, 16, 16, 16, 
    16, 16, 16, 16, 16, 16, 16, 16,
    16, 16, 16, 16, 16, 16, 16, 16
];

const ZIGZAG_ORDER: [i32; 64] = [
     0,  1,  8, 16,  9,  2,  3, 10, 
    17, 24, 32, 25, 18, 11,  4,  5, 
    12, 19, 26, 33, 40, 48, 41, 34, 
    27, 20, 13,  6,  7, 14, 21, 28, 
    35, 42, 49, 56, 57, 50, 43, 36, 
    29, 22, 15, 23, 30, 37, 44, 51, 
    58, 59, 52, 45, 38, 31, 39, 46, 
    53, 60, 61, 54, 47, 55, 62, 63
];

pub struct JPEGWriter {
}

impl JPEGWriter {

    pub fn new() -> Self {
        JPEGWriter {
        }
    }
}

impl ImageWriter for JPEGWriter {
    
    fn write(&self, image: &Image, _options: &ImageWriterOptions) -> Result<Vec<u8>, ImageIOError> {
        let mut quantization_tables: HashMap<u8, [i32; 64]> = HashMap::with_capacity(3);
        quantization_tables.insert(1, QUANTIZATION_TABLE_Y.clone());
        quantization_tables.insert(2, QUANTIZATION_TABLE_CB_CR.clone());
        
        let mut quantization_table_by_channel: HashMap<u8, u8> = HashMap::new();
        quantization_table_by_channel.insert(1, 1);
        quantization_table_by_channel.insert(2, 2);
        quantization_table_by_channel.insert(3, 2);

        let mut pixels_ycbcr: Vec<[i32; 3]> = Vec::with_capacity(image.width * image.height);
        for y in 0..image.height {
            for x in 0..image.width {
                pixels_ycbcr.push(rgb_to_ycbcr(&image.get_pixel(x, y)));
            }
        }

        // dct, quantization, zigzaging
        let mut channels: HashMap<u8, Vec<[i32; 64]>> = HashMap::with_capacity(3);
        for channel in 0..3 {
            let quantization_table = quantization_tables[&quantization_table_by_channel[&(channel + 1)]];

            let channel_values: Vec<i32> = pixels_ycbcr.iter().map(|v| v[channel as usize]).collect();
            let channel_values = split_into_mcus(channel_values).iter()
                .map(|mcu| dct_encode(&mcu))
                .map(|mcu| divide_64s(&mcu, &quantization_table))
                .map(|mcu| zigzag(&mcu))
                .collect();

            channels.insert(channel, channel_values);
        }

        // huffman encode

        Ok(vec![])
    }
}

fn zigzag(values: &[i32; 64]) -> [i32; 64] {
    let mut result = [0i32; 64];

    for i in 0..64 {
        result[i] = values[ZIGZAG_ORDER[i] as usize];
    }

    result
}

fn divide_64s(a: &[i32; 64], b: &[i32; 64]) -> [i32; 64] {
    let mut result = [0i32; 64];

    for i in 0..64 {
        result[i] = a[i] / b[i];
    }

    result
}

fn dct_encode(values: &[i32; 64]) -> [i32; 64] {
    let mut result = [0i32; 64];

    for v in 0..7 {
        for u in 0..7 {
            let mut sum = 0 as f32;

            for y in 0..7 {
                for x in 0..7 {
                    sum += values[y * 8 + x] as f32 
                        * (((2 * x + 1) * v) as f32 * PI / 16.0).cos() 
                        * (((2 * y + 1) * u) as f32 * PI / 16.0).cos();
                }
            }

            let cu = if u == 0 { 1.0/2f32.sqrt() } else { 1.0 };
            let cv = if v == 0 { 1.0/2f32.sqrt() } else { 1.0 };
            result[v * 8 + u] = (sum * cu * cv / 4.0).round() as i32;
        }
    }

    result
}

fn split_into_mcus(values: Vec<i32>) -> Vec<[i32; 64]> {
    let mut result: Vec<[i32; 64]> = Vec::new();
    let mut values = &values[..];

    while values.len() > 0 {
        if values.len() >= 64 {
            result.push(
                values[0..64].try_into()
                    .expect("expected to to get 64 byte array here because only 64 first bytes are taken")
            );
            values = &values[64..];
        } else {
            let mut entry = [0i32; 64];
            for i in 0..values.len() {
                entry[i] = values[i];
            }
            result.push(entry);
        }
    }

    result
}

fn rgb_to_ycbcr(pixel: &Pixel) -> [i32; 3] {
    [
        16 + (65.481 * pixel.red as f32 + 128.553 * pixel.green as f32 + 24.966 * pixel.blue as f32).round() as i32,
        128 + (-37.797 * pixel.red as f32 - 74.203 * pixel.green as f32 + 112.0 * pixel.blue as f32).round() as i32,
        128 + (112.0 * pixel.red as f32 - 93.786 * pixel.green as f32 + 18.214 * pixel.blue as f32).round() as i32
    ]
}

#[cfg(test)]
mod tests {
    use crate::reader::JPEGReader;

    use super::*;

    use core::models::io::{ImageReader, ImageWriterOptions};
    use std::fs::read;

    #[test]
    fn test_write_simple() {
        let image_data = read("assets/bridge.jpg")
            .expect("failed to load test image");
        
        let reader = JPEGReader::new();
        let images = reader.read(&image_data)
            .expect("failed to read test image");
        let image = &images[0];

        let writer = JPEGWriter::new();
        let new_image_data = writer.write(&image, &ImageWriterOptions::default())
            .expect("failed to write image");

        std::fs::write("assets/test.jpg", &new_image_data);
    }
}