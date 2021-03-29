use std::f32::consts::PI;

use core::models::{image::Image, pixel::Pixel, io::{ImageIOError, ImageWriter, ImageWriterOptions}};
use std::{collections::HashMap, convert::TryInto};

use byteorder::{BigEndian, ByteOrder};

use crate::common::{Channel, HuffmanTable, HuffmanTableType};

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
        quantization_tables.insert(0, QUANTIZATION_TABLE_Y.clone());
        quantization_tables.insert(1, QUANTIZATION_TABLE_CB_CR.clone());
        
        let mut quantization_table_by_channel: HashMap<u8, u8> = HashMap::new();
        quantization_table_by_channel.insert(1, 0);
        quantization_table_by_channel.insert(2, 1);
        quantization_table_by_channel.insert(3, 1);

        let mut pixels_ycbcr: Vec<[i32; 3]> = Vec::with_capacity(image.width * image.height);
        for y in 0..image.height {
            for x in 0..image.width {
                pixels_ycbcr.push(rgb_to_ycbcr(&image.get_pixel(x, y)));
            }
        }

        // dct, quantization, zigzaging
        let mut huffman_tables: Vec<HuffmanTable> = vec![
            HuffmanTable {
                id: 0,
                table_type: HuffmanTableType::DC,
                table: HashMap::new(),
            },
            HuffmanTable {
                id: 1,
                table_type: HuffmanTableType::DC,
                table: HashMap::new(),
            },
            HuffmanTable {
                id: 0,
                table_type: HuffmanTableType::AC,
                table: HashMap::new(),
            },
            HuffmanTable {
                id: 1,
                table_type: HuffmanTableType::DC,
                table: HashMap::new(),
            }
        ];
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

        let channels: Vec<Channel> = (0..3).map(|i| {
            let id  = i + 1;

            Channel {
                id,
                horizontal_sampling: 1,
                vertical_sampling: 1,
                quantization_table_id: *quantization_table_by_channel.get(&id).unwrap(),
            }
        }).collect();

        // huffman encode


        // writing
        let mut data = vec![0xFF, 0xD8]; // start with magic
        for (table_id, table) in quantization_tables {
            data.append(&mut prepend_marker(0xDB, write_quantization_table(table_id, &table)));
        }
        data.append(&mut prepend_marker(0xC0, write_baseline_dct(
            image.width as u16, 
            image.height as u16,
            &channels
        )));

        Ok(data)
    }
}

fn write_baseline_dct(width: u16, height: u16, channels: &Vec<Channel>) -> Vec<u8> {
    let mut data = vec![0u8; 17];
    
    let block_length = data.len();
    BigEndian::write_u16(&mut data[0..2], block_length as u16);

    // precision in bits for components
    data[2] = 8;

    BigEndian::write_u16(&mut data[3..5], height);
    BigEndian::write_u16(&mut data[5..7], width);

    let total_channels = 3;
    data[7] = total_channels;

    for i in 0..total_channels {
        let channel = &channels[i as usize];
        let offset: usize = 8 + (i as usize) * 3;

        data[offset] = channel.id;
        data[offset + 1] = channel.horizontal_sampling << 4 | channel.vertical_sampling;
        data[offset + 2] = channel.quantization_table_id;
    }

    data
}

fn prepend_marker(marker: u8, data: Vec<u8>) -> Vec<u8> {
    let mut data = data;
    let mut new_data = Vec::with_capacity(data.len() + 2);
    new_data.push(0xFF);
    new_data.push(marker);
    new_data.append(&mut data);
    new_data
}

fn write_quantization_table(table_id: u8, table: &[i32; 64]) -> Vec<u8> {
    let mut data = vec![0u8; 67];
    
    let data_length = data.len();
    BigEndian::write_u16(&mut data[0..2], data_length as u16);

    data[2] = table_id; // entry length is 0

    let zigzaged = zigzag(&table);
    for entry_index in 0..zigzaged.len() {
        data[entry_index + 3] = zigzaged[entry_index] as u8;
    }
    
    data
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
        
        info!("reading test image");
        let reader = JPEGReader::new();
        let images = reader.read(&image_data)
            .expect("failed to read test image");
        let image = &images[0];
        info!("done reading test image");

        let writer = JPEGWriter::new();
        let new_image_data = writer.write(&image, &ImageWriterOptions::default())
            .expect("failed to write image");

        std::fs::write("assets/test.jpg", &new_image_data);

        info!("reading new image");
        let new_images = reader.read(&new_image_data)
            .expect("failed to read new image");
        let new_image = &new_images[0];

        assert_eq!(image.pixels, new_image.pixels);
    }
}