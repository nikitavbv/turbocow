use std::f32::consts::PI;
use byteorder::{BigEndian, ByteOrder};
use custom_error::custom_error;
use lazy_static::lazy_static;
use bit_vec::BitVec;

use core::models::{image::Image, pixel::Pixel, io::{ImageIOError, ImageReader}};
use std::collections::HashMap;

use crate::{common::Channel, huffman::HuffmanTree};

// see:
// https://habr.com/ru/post/102521/

custom_error! {pub JPEGReaderError
    InvalidHeader {description: String} = "Invalid header: {description}",
    InvalidSegment {description: String} = "Invalid segment: {description}",
    InvalidEncodedData {description: String} = "Invalid encoded data: {description}",
}

lazy_static! {
    static ref DCT_PRECOMPUTED: [f32; 4096] = precompute_dct();
    static ref ZIGZAG_ORDER: [usize; 64] = [
        1,  2, 6,  7, 15, 16, 28, 29,
        3,  5, 8,  14, 17, 27, 30, 43,
        4,  9, 13, 18, 26, 31, 42, 44,
       10, 12, 19, 25, 32, 41, 45, 54,
       11, 20, 24, 33, 40, 46, 53, 55,
       21, 23, 34, 39, 47, 52, 56, 61,
       22, 35, 38, 48, 51, 57, 60, 62,
       36, 37, 49, 50, 58, 59, 63, 64 
    ];
}

pub struct JPEGReader {
}

impl JPEGReader {

    pub fn new() -> Self {
        JPEGReader {}
    }
}

#[derive(Clone)]
struct JPEG {

    image: Option<Image>,

    width: u16,
    height: u16,
    quantization_tables: Vec<QuantizationTable>,
    channels: Vec<Channel>,
    huffman_tables: Vec<HuffmanTable>,
}

impl JPEG {

    fn new() -> Self {
        JPEG {
            image: None,
            width: 0,
            height: 0,
            quantization_tables: Vec::new(),
            channels: Vec::new(),
            huffman_tables: Vec::new(),
        }
    }

    fn with_quantization_table(&self, table: QuantizationTable) -> Self {
        JPEG {
            image: self.image.clone(),
            width: self.width,
            height: self.height,
            quantization_tables: {
                let mut tables = self.quantization_tables.clone();
                tables.push(table);
                tables
            },
            channels: self.channels.clone(),
            huffman_tables: self.huffman_tables.clone(),
        }
    }

    fn with_baseline_dct(&self, width: u16, height: u16, channels: Vec<Channel>) -> Self {
        JPEG {
            image: self.image.clone(),
            width,
            height,
            quantization_tables: self.quantization_tables.clone(),
            channels,
            huffman_tables: self.huffman_tables.clone(),
        }
    }

    fn with_huffman_table(&self, table: HuffmanTable) -> Self {
        JPEG {
            image: self.image.clone(),
            width: self.width,
            height: self.height,
            quantization_tables: self.quantization_tables.clone(),
            channels: self.channels.clone(),
            huffman_tables: {
                let mut tables = self.huffman_tables.clone();
                tables.push(table);
                tables
            },
        }
    }

    fn with_image(&self, image: Image) -> Self {
        JPEG {
            image: Some(image),
            width: self.width,
            height: self.height,
            quantization_tables: self.quantization_tables.clone(),
            channels: self.channels.clone(),
            huffman_tables: self.huffman_tables.clone(),
        }
    }

    fn huffman_table_by_type(&self, table_type: HuffmanTableType, id: u8) -> Option<HuffmanTable> {
        self.huffman_tables.iter()
            .find(|t| t.table_type == table_type && t.id == id)
            .map(|v| v.clone())
    }

    fn quantization_table_by_id(&self, id: u8) -> Option<QuantizationTable> {
        self.quantization_tables.iter()
            .find(|t| t.id == id)
            .map(|v| v.clone())
    }
}

#[derive(Clone)]
struct QuantizationTable {
    id: u8,
    data: [i32; 64],
}

#[derive(Clone)]
struct HuffmanTable {

    id: u8,
    table_type: HuffmanTableType,
    table: HashMap<(u16, u16), u8>,
}

#[derive(Clone, PartialEq)]
enum HuffmanTableType {
    DC,
    AC
}

impl ImageReader for JPEGReader {
    
    fn read(&self, data: &Vec<u8>) -> Result<Vec<Image>, ImageIOError> {
        let magic = (data[0], data[1]);
        match magic {
            (0xFF, 0xD8) => {},
            other => panic!("Unexpected header magic: {:?}", other),
        };

        let mut jpeg = JPEG::new();
        let mut data = &data[2..];

        while jpeg.image.is_none() {
            let (image, offset) = read_segment(&data, &mut jpeg).expect("failed to read segment");
            jpeg = image;
            data = &data[offset..];
        }

        Ok(vec![jpeg.image.expect("expected image to be present, because checked for it previously")])
    }
}

fn read_segment(data: &[u8], jpeg: &JPEG) -> Result<(JPEG, usize), JPEGReaderError> {
    let marker = (data[0], data[1]);
    
    if marker.0 != 0xFF {
        return Err(JPEGReaderError::InvalidSegment {
            description: format!("Expected segment marker to start with 0xFF, instead got: {:x?}", marker.0),
        });
    }

   let data = &data[2..];

    match marker.1 {
        0xE0..=0xEF => read_application_specific_data(&data).map(|v| (jpeg.clone(), v)),
        0xFE => read_text_comment(&data).map(|v| (jpeg.clone(), v)),
        0xDB => read_quantization_table(&data).map(|v| (jpeg.with_quantization_table(v.0), v.1)),
        0xC0 => read_baseline_dct(&data, &jpeg),
        0xC4 => read_huffman_table(&data).map(|v| (jpeg.with_huffman_table(v.0), v.1)),
        0xDA => read_start_of_scan(&data, &jpeg),
        0xD9 => Ok((jpeg.clone(), 2)),
        _ => Err(JPEGReaderError::InvalidSegment {
            description: format!("Unknown segment marker: {:x?}", marker)
        })
    }
}

fn read_start_of_scan(data: &[u8], jpeg: &JPEG) -> Result<(JPEG, usize), JPEGReaderError> {
    trace!("reading start of scan");
    let block_length = BigEndian::read_u16(&data[0..2]) as usize;
    let data = &data[2..];

    let mut jpeg = jpeg.clone();
    
    let total_channels = data[0];
    if total_channels != 3 {
        return Err(JPEGReaderError::InvalidSegment {
            description: format!("Unexpected number of channels in start of scan segment: {}", total_channels),
        });
    }
    let mut data = &data[1..];

    let mut huffman_ac_by_channel: HashMap<u8, u8> = HashMap::new();
    let mut huffman_dc_by_channel: HashMap<u8, u8> = HashMap::new();

    for _ in 0..total_channels {
        let channel_id = data[0];
        let huffman_dc_id = data[1] >> 4;
        let huffman_ac_id = data[1] & 0b1111;
        data = &data[2..];

        huffman_dc_by_channel.insert(channel_id, huffman_dc_id);
        huffman_ac_by_channel.insert(channel_id, huffman_ac_id);
    }

    let _start_of_spectral_or_predictor_selection = data[0];
    let _end_of_spectral_selection = data[1];
    let _successive_approximation_bit_position = data[2];
    let data = &data[3..];

    let data = &data[0..data.len()-2];
    let mut new_data = Vec::with_capacity(data.len());
    let mut offset = 0;
    while offset < data.len() {
        let byte = data[offset];
        if byte == 0xFF {
            if data[offset + 1] == 0x00 {
                offset += 1;
            } else if data[offset + 1] == 0xD9 {
                break;
            } else {
                return Err(JPEGReaderError::InvalidEncodedData {
                    description: format!("Unexpected marker in encoded data: {:x?} {:x?}", data[offset], data[offset + 1])
                });
            }
        }
        new_data.push(byte);

        offset += 1;
    }
    let data_length = offset;
    let data: &[u8] = &new_data;

    // reading encoded bits
    let bitvec = BitVec::from_bytes(&data);
    let mut offset = 0;

    trace!("bitvec length is {}", bitvec.len());

    trace!("image dimensions: {} {}", jpeg.width, jpeg.height);
    let max_horizontal_sampling = 8 * jpeg.channels.iter().map(|c| c.horizontal_sampling).max()
        .expect("expected at least one channel to be present");
    let max_vertical_sampling = 8 * jpeg.channels.iter().map(|c| c.vertical_sampling).max()
        .expect("expected at least one channel to be present");
    trace!("max sampling: {} {}", max_horizontal_sampling, max_vertical_sampling);
    let horizontal_mcus = ((jpeg.width as f32) / (max_horizontal_sampling as f32)).ceil() as usize;
    let vertical_mcus = ((jpeg.height as f32) / (max_vertical_sampling as f32)).ceil() as usize;
    trace!("image dimensions in MCUs: {} {}", horizontal_mcus, vertical_mcus);
    let mut prev_dc = vec![0; total_channels as usize];

    let mut image = Image::new(
        ((jpeg.width as f32 / max_horizontal_sampling as f32).ceil() * max_horizontal_sampling as f32) as usize, 
        ((jpeg.height as f32 / max_vertical_sampling as f32).ceil() * max_vertical_sampling as f32) as usize, 
    );

    for row in 0..vertical_mcus {
        for col in 0..horizontal_mcus {
            let mut channel_data: HashMap<u8, Vec<i32>> = HashMap::new();

            for channel_id in 1..=total_channels {
                let channel = jpeg.channels.iter().find(|c| c.id == channel_id).ok_or(JPEGReaderError::InvalidEncodedData {
                    description: format!("Channel with id {} not found", channel_id),
                })?;
                let mut matrices: Vec<[i32; 64]> = Vec::new();

                let mut bitgroup = 0;
                let mut bitgroup_length = 0;            

                let dc_huffman_table = jpeg.huffman_table_by_type(HuffmanTableType::DC, huffman_dc_by_channel[&(channel_id as u8)])
                    .ok_or(JPEGReaderError::InvalidEncodedData {
                        description: format!("DC Huffman table with id = {} is not present", channel_id)
                    })?.table;
                let ac_huffman_table = jpeg.huffman_table_by_type(HuffmanTableType::AC ,huffman_ac_by_channel[&(channel_id as u8)])
                    .ok_or(JPEGReaderError::InvalidEncodedData {
                        description: format!("DC Huffman table with id = {} is not present", channel_id)
                    })?.table;

                for _ in 0..channel.vertical_sampling {
                    for _ in 0..channel.horizontal_sampling {
                        let mut dc_factor_read = false;
                        let mut factor_vals: [i32; 64] = [0i32; 64];
                        let mut factor_offset = 0;

                        while factor_offset < 64 {
                            bitgroup = (bitgroup << 1) | (if bitvec[offset] { 1 } else { 0 });
                            offset += 1;
                            bitgroup_length += 1;

                            if !dc_factor_read {
                                if dc_huffman_table.contains_key(&(bitgroup, bitgroup_length)) {
                                    let value = dc_huffman_table[&(bitgroup, bitgroup_length)];
                                    bitgroup = 0;
                                    bitgroup_length = 0;
                    
                                    if value == 0 {
                                        factor_offset += 1;
                                    } else {
                                        let mut factor: i32 = 0;
                                        let mut first_bit_is_one = false;
                                        for i in 0..value {
                                            if i == 0 {
                                                first_bit_is_one = bitvec[offset];
                                            }
                                            
                                            factor = (factor << 1) | (if bitvec[offset] { 1 } else { 0 });
                                            offset += 1;
                                        }
                    
                                        if !first_bit_is_one {
                                            factor = factor - 2i32.pow(value as u32) + 1;
                                        }

                                        factor_vals[factor_offset] = factor;
                                        factor_offset += 1;
                                    }
                                    dc_factor_read = true;
                                }
                            } else {
                                if ac_huffman_table.contains_key(&(bitgroup, bitgroup_length)) {
                                    let value = ac_huffman_table[&(bitgroup, bitgroup_length)];
                                    bitgroup = 0;
                                    bitgroup_length = 0;
                    
                                    if value == 0 {
                                        factor_offset = 64;
                                    } else {
                                        let number_of_zeros = value >> 4;
                                        let factor_length = value & 0b1111;
                                        factor_offset += number_of_zeros as usize;
                    
                                        let mut factor: i32 = 0;
                                        let mut first_bit_is_one = false;
                                        for i in 0..factor_length {
                                            if i == 0 {
                                                first_bit_is_one = bitvec[offset];
                                            }
                                            
                                            factor = (factor << 1) | (if bitvec[offset] { 1 } else { 0 });
                                            offset += 1;
                                        }
                    
                                        if !first_bit_is_one {
                                            factor = factor - 2i32.pow(factor_length as u32) + 1;
                                        }
                    
                                        factor_vals[factor_offset] = factor;
                                        factor_offset += 1;
                                    }
                                }
                            }

                            if factor_offset == 64 {
                                factor_vals[0] += prev_dc[channel_id as usize - 1];
                                prev_dc[channel_id as usize - 1] = factor_vals[0];
                                matrices.push(unzigzag_64(&factor_vals));
                            }
                        }
                    }
                }

                // quantization
                let quantization_table: [i32; 64] = jpeg.quantization_table_by_id(channel.quantization_table_id)
                    .ok_or(JPEGReaderError::InvalidEncodedData {
                        description: format!("Quantization table with id {} not found", channel.quantization_table_id)
                    })?
                    .data;
                
                // discrete cosine transform
                let matrices: Vec<[i32; 64]> = matrices.iter()
                    .map(|m| multiply_64(&m, &quantization_table))
                    .map(|m| perform_dct(&m))
                    .collect();

                // how many pixels should each unit take
                let v_ratio = max_vertical_sampling / channel.vertical_sampling;
                let h_ratio = max_horizontal_sampling / channel.horizontal_sampling;
                // scaling (i.e. 1 unit pixel = x actual pixels)
                let v_scaling = v_ratio / 8;
                let h_scaling = h_ratio / 8;

                let mut scale_result: Vec<i32> = vec![0; max_horizontal_sampling as usize * max_vertical_sampling as usize];
                let mut offset_x: usize = 0;
                let mut offset_y: usize = 0;
                for matrix_index in 0..matrices.len() {
                    let matrix = &matrices[matrix_index];
                                                            
                    for y in 0..8 {
                        for x in 0..8 {
                            let value = matrix[y * 8 + x];

                            for hs in 0..h_scaling {
                                for vs in 0..v_scaling {
                                    let pos = (y * v_scaling as usize + vs as usize + offset_y) * max_horizontal_sampling as usize + 
                                        (x * h_scaling as usize + hs as usize + offset_x);
                                    scale_result[pos] = value;
                                }
                            }
                        }
                    }

                    offset_x += h_ratio as usize;
                    if offset_x == max_horizontal_sampling as usize {
                        offset_x = 0;
                        offset_y += v_ratio as usize;
                    }
                }

                channel_data.insert(channel_id, scale_result);
            }

            let image_x_offset = col * max_horizontal_sampling as usize;
            let image_y_offset = row * max_vertical_sampling as usize;

            let y_channel = channel_data.get(&1).expect("Expected channel with id = 1 to be present, becuase already checked for that");
            let cb_channel = channel_data.get(&2).expect("Expected channel with id = 2 to be present, becuase already checked for that");
            let cr_channel = channel_data.get(&3).expect("Expected channel with id = 3 to be present, becuase already checked for that");
            for y in 0..max_vertical_sampling {
                for x in 0..max_horizontal_sampling {
                    let (r, g, b) = ycbcr_to_rgb(
                        y_channel[(y * max_horizontal_sampling + x) as usize], 
                        cb_channel[(y * max_horizontal_sampling + x) as usize], 
                        cr_channel[(y * max_horizontal_sampling + x) as usize]
                    );

                    image.set_pixel(image_x_offset + x as usize, image_y_offset + y as usize, Pixel::from_rgb(r, g, b));
                }
            }

        }
    }

    jpeg = jpeg.with_image(image);
    Ok((jpeg, block_length + 2 + data_length))
}

fn ycbcr_to_rgb(y: i32, cb: i32, cr: i32) -> (u8, u8, u8) {
    let r = ((y as f32 + 1.402 * (cr as f32 - 128.0)).round() as i32).max(0).min(255) as u8;
    let g = ((y as f32 - 0.34414 * (cb as f32 - 128.0) - 0.71414 * (cr as f32 - 128.0)).round() as i32).max(0).min(255) as u8;
    let b = ((y as f32 + 1.772 * (cb as f32 - 128.0)).round() as i32).max(0).min(255) as u8;
    (r, g, b)
}

fn precompute_dct() -> [f32; 4096] {
    let mut res = [0f32; 4096];

    for y in 0..8 {
        for x in 0..8 {
            for v in 0..8 {
                for u in 0..8 {
                    let mut m: f32 = if u == 0 { 1.0/2f32.sqrt() } else { 1.0 };
                    m *= if v == 0 { 1.0/2f32.sqrt() } else { 1.0 };
                    m *= (((2.0 * (x as f32) + 1.0) * (u as f32) * PI) / 16.0).cos();
                    m *= (((2.0 * (y as f32) + 1.0) * (v as f32) * PI) / 16.0).cos();
                    res[y * 512 + x * 64 + v * 8 + u] = m / 4.0;
                }
            }
        }
    }

    res
}

fn perform_dct(matrix: &[i32; 64]) -> [i32; 64] {
    let mut result  = [0f32; 8 * 8];

    for y in 0..8 {
        for x in 0..8 {
            let mut sum: f32 = 0.0;
            let offset = y * 512 + x * 64;

            for (a, b) in matrix.iter().zip(&DCT_PRECOMPUTED[offset..offset+64]) {
                sum += *a as f32 * *b;
            }

            result[y * 8 + x] = sum;
        }
    }

    let mut result_rounded = [128; 64];
    for i in 0..64 {
        result_rounded[i] += (result[i] as i32).max(-128).min(128);
    }

    result_rounded
}

fn unzigzag_64(matrix: &[i32; 64]) -> [i32; 64] {
    let mut result = [0i32; 64];

    for i in 0..64 {
        result[i] = matrix[ZIGZAG_ORDER[i] - 1];
    }

    result
}

fn multiply_64(a: &[i32; 64], b: &[i32; 64]) -> [i32; 64] {
    let mut result = [0i32; 64];

    for i in 0..64 {
        result[i] = a[i] * b[i];
    }

    result
}

fn read_huffman_table(data: &[u8]) -> Result<(HuffmanTable, usize), JPEGReaderError> {
    let block_length = BigEndian::read_u16(&data[0..2]) as usize;
    let data = &data[2..];

    let packed = data[0];
    let table_type = match packed >> 4 {
        0 => HuffmanTableType::DC,
        1 => HuffmanTableType::AC,
        other => return Err(JPEGReaderError::InvalidSegment {
            description: format!("Unexpected huffman table type: {}", other)
        })
    };
    let table_id = packed & 0b1111;
    let data = &data[1..(block_length - 2)];

    let mut tree = HuffmanTree::new();
    let mut offset = 16;
    for code_length in 1..=16 {
        for _ in 0..data[code_length - 1] {
            tree.insert_code(code_length as u8, data[offset]);
            offset += 1;
        }
    }

    Ok((HuffmanTable {
        id: table_id,
        table_type: table_type,
        table: tree.to_map(),
    }, block_length + 2))
}

fn read_baseline_dct(data: &[u8], jpeg: &JPEG) -> Result<(JPEG, usize), JPEGReaderError> {
    let block_length = BigEndian::read_u16(&data[0..2]) as usize;
    let data = &data[2..];

    // preicion in bits for components
    let precision = data[0];
    if precision != 8 {
        return Err(JPEGReaderError::InvalidSegment {
            description: format!("Only precision = 8 is supported, got = {}", precision),
        });
    }

    let height = BigEndian::read_u16(&data[1..3]); // number of lines
    let width = BigEndian::read_u16(&data[3..5]); // samples per line
    trace!("dimensions: {} {}", width, height);

    let total_channels: u8 = data[5];
    if total_channels != 3 {
        return Err(JPEGReaderError::InvalidSegment {
            description: format!("Unsupported number of channels: {}", total_channels),
        });
    }

    let mut data = &data[6..];
    let mut channels = Vec::new();
    for _ in 0..total_channels {
        let channel_id = data[0];
        let horizontal_sampling = data[1] >> 4;
        let vertical_sampling = data[1] & 0b1111;
        let quantization_table_id = data[2];
        data = &data[3..];

        trace!(
            "channel: id={} horizontal_sampling={} vertical_sampling={} quantization_table={}", 
            channel_id, 
            horizontal_sampling, 
            vertical_sampling, 
            quantization_table_id
        );

        channels.push(Channel {
            id: channel_id,
            horizontal_sampling,
            vertical_sampling,
            quantization_table_id
        });
    }

    Ok((jpeg.with_baseline_dct(width, height, channels), block_length + 2))
}

fn read_quantization_table(data: &[u8]) -> Result<(QuantizationTable, usize), JPEGReaderError> {
    trace!("reading quantization table");

    let block_length = BigEndian::read_u16(&data[0..2]) as usize;
    let data = &data[2..];

    let packed = data[0];
    let entry_length = packed >> 4;
    let table_id = packed & 0b1111;
    let data = &data[1..];

    if entry_length != 0 {
        return Err(JPEGReaderError::InvalidSegment {
            description: format!("Quantization tables with entries of length {} are not supported", entry_length),
        });
    }
    
    let mut new_data = [0i32; 64];
    for i in 0..64 {
        new_data[i] = data[i] as i32;
    }

    let data = unzigzag_64(&new_data);

    let table = QuantizationTable {
        id: table_id,
        data,
    };

    Ok((table, block_length + 2))
}

fn read_text_comment(data: &[u8]) -> Result<usize, JPEGReaderError> {
    Ok(BigEndian::read_u16(&data[0..2]) as usize + 2)
}

fn read_application_specific_data(data: &[u8]) -> Result<usize, JPEGReaderError> {
    Ok(BigEndian::read_u16(&data[0..2]) as usize + 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read;

    #[ctor::ctor]
    fn init() {
        env_logger::init();
    }

    #[test]
    fn test_read_simple() {
        let image_data = read("assets/bridge.jpg")
            .expect("failed to load test image");
        
        let reader = JPEGReader::new();
        let images = reader.read(&image_data)
            .expect("failed to read test image");

        assert_eq!(images.len(), 1);

        let image = &images[0];
        assert_eq!(Pixel::from_rgb(109, 115, 127), image.get_pixel(720, 700));
        assert_eq!(Pixel::from_rgb(216, 148, 169), image.get_pixel(1290, 550));
    }
}