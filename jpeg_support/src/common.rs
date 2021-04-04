use std::collections::HashMap;

use bit_vec::BitVec;

use core::models::pixel::Pixel;

use crate::errors::JPEGReaderError;

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

const UNZIGZAG_ORDER: [usize; 64] = [
    1,  2, 6,  7, 15, 16, 28, 29,
    3,  5, 8,  14, 17, 27, 30, 43,
    4,  9, 13, 18, 26, 31, 42, 44,
   10, 12, 19, 25, 32, 41, 45, 54,
   11, 20, 24, 33, 40, 46, 53, 55,
   21, 23, 34, 39, 47, 52, 56, 61,
   22, 35, 38, 48, 51, 57, 60, 62,
   36, 37, 49, 50, 58, 59, 63, 64 
];

pub type ChannelID = u8;
pub type HuffmanTableID = u8;

#[derive(Clone, Debug)]
pub struct Channel {
    pub id: ChannelID,
    pub horizontal_sampling: u8,
    pub vertical_sampling: u8,
    pub quantization_table_id: u8,
}

#[derive(Clone, Debug)]
pub struct HuffmanTable {

    pub id: HuffmanTableID,
    pub table_type: HuffmanTableType,
    pub table: HashMap<(u16, u16), u8>,
}

impl HuffmanTable {

    pub fn from_vk(id: HuffmanTableID, table_type: HuffmanTableType, vk: &HashMap<u8, (u16, u16)>) -> Self {
        let mut table = HashMap::new();

        for (v, k) in vk {
            table.insert(k.clone(), *v);
        }

        Self {
            id,
            table_type,
            table,
        }
    }

    pub fn vk_table(&self) -> HashMap<u8, (u16, u16)> {
        let mut table = HashMap::new();

        for (k, v) in &self.table {
            table.insert(*v, k.clone());
        }

        table
    }
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

pub fn zigzag(values: &[i32; 64]) -> [i32; 64] {
    let mut result = [0i32; 64];

    for i in 0..64 {
        result[i] = values[ZIGZAG_ORDER[i] as usize];
    }

    result
}

pub fn unzigzag_64(matrix: &[i32; 64]) -> [i32; 64] {
    let mut result = [0i32; 64];

    for i in 0..64 {
        result[i] = matrix[UNZIGZAG_ORDER[i] - 1];
    }

    result
}

pub fn write_huffman_encoded_channels_data(
    data: &Vec<[[i32; 64]; 3]>,
    huffman_tables: &HashMap<(HuffmanTableType, ChannelID), HuffmanTable>,
) -> Vec<u8> {
    let mut result = Vec::new();
    let mut prev_dc = [0i32; 3];

    for mcu in data {
        for channel_id in 1..=(mcu.len() as ChannelID) {
            let channel = mcu[channel_id as usize - 1];
            let dc_huffman_table: &HashMap<u8, (u16, u16)> = &huffman_tables[&(HuffmanTableType::DC, channel_id)].vk_table();
            let ac_huffman_table: &HashMap<u8, (u16, u16)> = &huffman_tables[&(HuffmanTableType::AC, channel_id)].vk_table();

            let mut channel = zigzag(&channel);
                
            channel[0] = channel[0] - prev_dc[channel_id as usize - 1];
            prev_dc[channel_id as usize - 1] = channel[0];

            let mut block_data = BitVec::new();
            // trace!("writing dc value: {}", channel[0]);
            write_factor(&mut block_data, dc_huffman_table, channel[0]);
            for i in 1..64 {
                let ac = channel[i];
                write_factor(&mut block_data, ac_huffman_table, ac);
            }

            result.append(&mut block_data.to_bytes());
        }
    }

    result
}

fn write_factor(output_bitvec: &mut BitVec, huffman_table: &HashMap<u8, (u16, u16)>, factor: i32) {
    //trace!("writing factor: {}", factor);
    let non_zero_digits = count_non_zero_digits(factor) + 1;

    // TODO: negative numbers?

    write_huffman_code(output_bitvec, huffman_table[&non_zero_digits]);
    if non_zero_digits > 0 {
        write_number_bits(output_bitvec, factor, non_zero_digits);
    }
}

fn write_huffman_code(output_bitvec: &mut BitVec, code: (u16, u16)) {
    write_number_bits(output_bitvec, code.0 as i32, code.1 as u8)
}

fn write_number_bits(output_bitvec: &mut BitVec, number: i32, total_bits: u8) {
    // trace!("writing huffman with {} {}", number, total_bits);
    for i in 0..total_bits {
        let index = total_bits - i;
        output_bitvec.push(((number >> index) & 0b1) == 1);
    }
}

fn count_non_zero_digits(value: i32) -> u8 {
    let mut result = 0;

    for i in 0..32 {
        if value & (1 << i) == 1 {
            result = i;
        }
    }

    result
}

pub fn read_huffman_encoded_channels_data(
    bitvec: &BitVec, 
    total_mcus: usize, 
    channels: &HashMap<ChannelID, Channel>, 
    huffman_tables_by_channel: &HashMap<(HuffmanTableType, ChannelID), HuffmanTable>,
) -> Result<Vec<[i32; 64]>, JPEGReaderError> {
    let total_channels = channels.len();

    let mut result = Vec::new();
    let mut prev_dc = vec![0; total_channels as usize];
    let mut offset = 0;

    for _ in 0..total_mcus {
        for channel_id in 1..=(total_channels as ChannelID) {
            let channel = &channels[&channel_id];

            let dc_huffman_table: &HashMap<(u16, u16), u8> = &huffman_tables_by_channel[&(HuffmanTableType::DC, channel_id)].table;
            let ac_huffman_table: &HashMap<(u16, u16), u8> = &huffman_tables_by_channel[&(HuffmanTableType::AC, channel_id)].table;

            let mut bitgroup = 0;
            let mut bitgroup_length = 0;

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
                                // trace!("reading db by {:?}", (bitgroup, bitgroup_length));
                                bitgroup = 0;
                                bitgroup_length = 0;
                
                                if value == 0 {
                                    factor_offset += 1;
                                    // trace!("dc is 0");
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

                                    // trace!("read dc value: {}", factor);
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
                
                                    //trace!("read ac value: {}", factor);
                                    factor_vals[factor_offset] = factor;
                                    factor_offset += 1;
                                }
                            }
                        }

                        if factor_offset == 64 {
                            factor_vals[0] += prev_dc[channel_id as usize - 1];
                            prev_dc[channel_id as usize - 1] = factor_vals[0];
                            result.push(unzigzag_64(&factor_vals));
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}