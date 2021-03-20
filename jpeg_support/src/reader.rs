use std::f32::consts::PI;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use custom_error::custom_error;
use bit_vec::BitVec;

use core::models::{image::Image, pixel::Pixel, io::{ImageIOError, ImageReader}};
use std::collections::HashMap;

use crate::huffman::HuffmanTree;

// see:
// https://habr.com/ru/post/102521/

custom_error! {pub JPEGReaderError
    InvalidHeader {description: String} = "Invalid header: {description}",
    InvalidSegment {description: String} = "Invalid segment: {description}",
    InvalidEncodedData {description: String} = "Invalid encoded data: {description}",
}

pub struct JPEGReader {
}

impl JPEGReader {

    fn new() -> Self {
        JPEGReader {}
    }
}

#[derive(Clone)]
struct JPEG {

    width: u16,
    height: u16,
    quantization_tables: Vec<QuantizationTable>,
    channels: Vec<Channel>,
    huffman_tables: Vec<HuffmanTable>,
    ready: bool,
}

impl JPEG {

    fn new() -> Self {
        JPEG {
            width: 0,
            height: 0,
            quantization_tables: Vec::new(),
            channels: Vec::new(),
            huffman_tables: Vec::new(),
            ready: false,
        }
    }

    fn with_quantization_table(&self, table: QuantizationTable) -> Self {
        JPEG {
            width: self.width,
            height: self.height,
            quantization_tables: {
                let mut tables = self.quantization_tables.clone();
                tables.push(table);
                tables
            },
            channels: self.channels.clone(),
            huffman_tables: self.huffman_tables.clone(),
            ready: false,
        }
    }

    fn with_baseline_dct(&self, width: u16, height: u16, channels: Vec<Channel>) -> Self {
        JPEG {
            width,
            height,
            quantization_tables: self.quantization_tables.clone(),
            channels,
            huffman_tables: self.huffman_tables.clone(),
            ready: false,
        }
    }

    fn with_huffman_table(&self, table: HuffmanTable) -> Self {
        JPEG {
            width: self.width,
            height: self.height,
            quantization_tables: self.quantization_tables.clone(),
            channels: self.channels.clone(),
            huffman_tables: {
                let mut tables = self.huffman_tables.clone();
                tables.push(table);
                tables
            },
            ready: false,
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
    data: Vec<u8>, // raw
}

#[derive(Clone)]
struct Channel {
    id: u8,
    horizontal_sampling: u8,
    vertical_sampling: u8,
    quantization_table_id: u8,
}

#[derive(Clone)]
struct HuffmanTable {

    id: u8,
    table_type: HuffmanTableType,
    table: HashMap<u16, u8>,
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

        while !jpeg.ready {
            let (image, offset) = read_segment(&data, &mut jpeg).expect("failed to read segment");
            jpeg = image;
            data = &data[offset..];
        }

        Ok(vec![])
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
        0xDA => read_start_of_scan(&data, &jpeg).map(|v| (jpeg.clone(), v)),
        0xD9 => {
            let mut jpg = jpeg.clone();
            jpg.ready = true;
            Ok((jpg, 0))
        },
        _ => Err(JPEGReaderError::InvalidSegment {
            description: format!("Unknown segment marker: {:x?}", marker)
        })
    }
}

fn read_start_of_scan(data: &[u8], jpeg: &JPEG) -> Result<usize, JPEGReaderError> {
    trace!("reading start of scan");
    let block_length = BigEndian::read_u16(&data[0..2]) as usize;
    let data = &data[2..];
    
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
    let mut bitgroup = 0;

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

    for row in 0..vertical_mcus {
        trace!("reading row {}/{} with offset {}", row, vertical_mcus, offset);

        for col in 0..horizontal_mcus {
            for channel_id in 1..=total_channels {
                println!("channel, offset is {}", offset);

                let channel = jpeg.channels.iter().find(|c| c.id == channel_id).unwrap();
                let mut prev_dc = None;
                let mut matrices: Vec<Vec<i32>> = Vec::new();

                let dc_huffman_table = jpeg.huffman_table_by_type(HuffmanTableType::DC, huffman_dc_by_channel[&(channel_id as u8)])
                    .ok_or(JPEGReaderError::InvalidEncodedData {
                        description: format!("DC Huffman table with id = {} is not present", channel_id)
                    })?.table;
                let ac_huffman_table = jpeg.huffman_table_by_type(HuffmanTableType::AC ,huffman_ac_by_channel[&(channel_id as u8)])
                    .ok_or(JPEGReaderError::InvalidEncodedData {
                        description: format!("DC Huffman table with id = {} is not present", channel_id)
                    })?.table;

                for unit_row in 0..channel.vertical_sampling {
                    for unit_row in 0..channel.horizontal_sampling {
                        println!("dataunit, offset is {}", offset);

                        let mut dc_factor_read = false;
                        let mut factors: Vec<i32> = Vec::with_capacity(8 * 8);

                        while factors.len() < 64 {
                            bitgroup = (bitgroup << 1) | (if bitvec[offset] { 1 } else { 0 });
                            offset += 1;

                            if !dc_factor_read {
                                if dc_huffman_table.contains_key(&bitgroup) {
                                    let value = dc_huffman_table[&bitgroup];
                                    bitgroup = 0;
                    
                                    if value == 0 {
                                        factors.push(0);
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
                    
                                        factors.push(factor);
                                    }
                                    dc_factor_read = true;
                                }
                            } else {
                                if ac_huffman_table.contains_key(&bitgroup) {
                                    let value = ac_huffman_table[&bitgroup];
                                    //trace!("read huffman: {} by key {}", value, bitgroup);
                                    bitgroup = 0;
                    
                                    if value == 0 {
                                        while factors.len() < 64 {
                                            factors.push(0);
                                        }
                                    } else {
                                        let number_of_zeros = value >> 4;
                                        let factor_length = value & 0b1111;
                    
                                        for _ in 0..number_of_zeros {
                                            factors.push(0);
                                        }
                    
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
                    
                                        factors.push(factor);
                                    }
                                }
                            }

                            if factors.len() == 64 {
                                if let Some(prev_dc) = prev_dc {
                                    factors[0] += prev_dc;
                                }
                    
                                prev_dc = Some(factors[0]);
                                matrices.push(factors.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    
    // ----
    /*
    while offset < bitvec.len() {

        
    }

    if (y_matrices.len() as u16) < y_matrices_expected {
        return Err(JPEGReaderError::InvalidEncodedData {
            description: format!("Read less y marices then expected: {} < {}", y_matrices.len(), y_matrices_expected),
        });
    }

    if (cb_matrices.len() as u16) < cb_matrices_expected {
        return Err(JPEGReaderError::InvalidEncodedData {
            description: format!("Read less cb marices then expected: {} < {}", cb_matrices.len(), cb_matrices_expected),
        });
    }

    if (cr_matrices.len() as u16) < cr_matrices_expected {
        return Err(JPEGReaderError::InvalidEncodedData {
            description: format!("Read less cr marices then expected: {} < {}", cr_matrices.len(), cr_matrices_expected),
        });
    }

    // quantization
    trace!("total quantization tables: {}", jpeg.quantization_tables.len());
    let quantization_table: Vec<i32> = jpeg.quantization_table_by_id(0).ok_or(JPEGReaderError::InvalidEncodedData {
        description: "quantization matrix with id = 0 not found".to_string(),
    })?.data.iter().map(|v| *v as i32).collect();
    let y_matrices: Vec<Vec<i32>> = y_matrices.iter()
        .map(|matrix| multiply(&matrix, &quantization_table))
        .collect();

    let quantization_table: Vec<i32> = jpeg.quantization_table_by_id(1).ok_or(JPEGReaderError::InvalidEncodedData {
        description: "quantization matrix with id = 1 not found".to_string(),
    })?.data.iter().map(|v| *v as i32).collect();
    let cb_matrices: Vec<Vec<i32>> = cb_matrices.iter()
        .map(|matrix| multiply(&matrix, &quantization_table))
        .collect();
    let cr_matrices: Vec<Vec<i32>> = cr_matrices.iter()
        .map(|matrix| multiply(&matrix, &quantization_table))
        .collect();

    // discrete cosine transform
    let y_matrices: Vec<Vec<i32>> = y_matrices.iter()
        .map(|m| unzigzag(m))
        .map(|m| perform_dct(&m))
        .collect();
    let cb_matrices: Vec<Vec<i32>> = cb_matrices.iter()
        .map(|m| unzigzag(m))
        .map(|m| perform_dct(&m))
        .collect();
    let cr_matrices: Vec<Vec<i32>> = cr_matrices.iter()
        .map(|m| unzigzag(m))
        .map(|m| perform_dct(&m))
        .collect();
    
    // to rgb
    let mut image = Image::new(8,8);
    for y in 0..8 {
        for x in 0..8 {
            let (r, g, b) = ycbcr_to_rgb(y_matrices[0][y * 8 + x], cb_matrices[0][y * 8 / 2 + x / 2], cr_matrices[0][y * 8 / 2 + x / 2]);
            image.set_pixel(x, y, Pixel::from_rgb(r, g, b));
        }
    }*/

    Ok(block_length + 2 + data_length)
}

fn ycbcr_to_rgb(y: i32, cb: i32, cr: i32) -> (u8, u8, u8) {
    let r = ((y as f32 + 1.402 * (cr as f32 - 128.0)).round() as i32).max(0).min(255) as u8;
    let g = ((y as f32 - 0.34414 * (cb as f32 - 128.0) - 0.71414 * (cr as f32 - 128.0)).round() as i32).max(0).min(255) as u8;
    let b = ((y as f32 + 1.772 * (cb as f32 - 128.0)).round() as i32).max(0).min(255) as u8;

    (r, g, b)
}

fn perform_dct(matrix: &[i32]) -> Vec<i32> {
    let mut result: Vec<i32> = vec![0; 8 * 8];

    for y in 0..8 {
        for x in 0..8 {
            let mut sum: f32 = 0.0;

            for u in 0..8 {
                for v in 0..8 {
                    let mut m: f32 = if u == 0 { 1.0/2f32.sqrt() } else { 1.0 };
                    m *= if v == 0 { 1.0/2f32.sqrt() } else { 1.0 };
                    m *= matrix[v * 8 + u] as f32;
                    m *= (((2.0 * (x as f32) + 1.0) * (u as f32) * PI) / 16.0).cos();
                    m *= (((2.0 * (y as f32) + 1.0) * (v as f32) * PI) / 16.0).cos();

                    sum += m;
                }
            }

            result[y * 8 + x] = ((sum / 4.0).round() as i32 + 128).max(0).min(255);
        }
    }

    result
}

fn unzigzag(matrix: &[i32]) -> Vec<i32> {
    // I am so lazy today, lol
    let order = vec![
         1,  2, 6,  7, 15, 16, 28, 29,
         3,  5, 8,  14, 17, 27, 30, 43,
         4,  9, 13, 18, 26, 31, 42, 44,
        10, 12, 19, 25, 32, 41, 45, 54,
        11, 20, 24, 33, 40, 46, 53, 55,
        21, 23, 34, 39, 47, 52, 56, 61,
        22, 35, 38, 48, 51, 57, 60, 62,
        36, 37, 49, 50, 58, 59, 63, 64 
    ];

    let mut result = Vec::with_capacity(matrix.len());

    for i in 0..matrix.len() {
        result.push(matrix[order[i] - 1]);
    }

    result
}

fn multiply(a: &Vec<i32>, b: &Vec<i32>) -> Vec<i32> {
    let mut result = Vec::with_capacity(a.len());

    for i in 0..a.len() {
        result.push(a[i] * b[i]);
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

    let table = QuantizationTable {
        id: table_id,
        data: data.to_vec(),
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
        let image_data = read("assets/google.jpg")
            .expect("failed to load test image");
        
        let reader = JPEGReader::new();
        let images = reader.read(&image_data)
            .expect("failed to read test image");

        //assert_eq!(images.len(), 1);
    }
}