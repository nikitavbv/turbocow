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
    let mut block_data = BitVec::new();
    let mut prev_dc = [0i32; 3];

    for mcu in data {
        for channel_id in 1..=(mcu.len() as ChannelID) {
            let channel = mcu[channel_id as usize - 1];
            let dc_huffman_table = &huffman_tables[&(HuffmanTableType::DC, channel_id)];
            let ac_huffman_table = &huffman_tables[&(HuffmanTableType::AC, channel_id)];

            write_huffman_encoded_matrix(&mut block_data, &mut prev_dc, &dc_huffman_table, &ac_huffman_table, channel_id, &channel);
        }
    }

    while block_data.len() % 8 != 0 {
        block_data.push(true);
    }

    block_data.to_bytes()
}

fn write_huffman_encoded_matrix(
    block_data: &mut BitVec, 
    prev_dc: &mut [i32; 3],
    dc_huffman_table: &HuffmanTable,
    ac_huffman_table: &HuffmanTable,
    channel_id: ChannelID, 
    channel: &[i32; 64]
) {
    let dc_huffman_table = dc_huffman_table.vk_table();
    let ac_huffman_table = ac_huffman_table.vk_table();

    //println!("write matrix: {:?}", channel);
    let mut channel = zigzag(&channel);

    let prev_value = channel[0];
    channel[0] = channel[0] - prev_dc[channel_id as usize - 1];
    prev_dc[channel_id as usize - 1] = prev_value;

    write_factor(block_data, &dc_huffman_table, channel[0], 0);

    let mut offset = 1;
    while offset < 64 {
        let mut following_zeros = 0;
        while offset + following_zeros < 64 {
            if channel[offset + following_zeros] == 0 {
                following_zeros += 1;
            } else {
                break;
            }
        }

        if offset + following_zeros == 64 {
            write_factor(block_data, &ac_huffman_table, 0, 0);
            break;
        }

        following_zeros = following_zeros.min(15);
        offset += following_zeros;

        let ac = channel[offset];

        write_factor(block_data, &ac_huffman_table, ac, following_zeros as u8);

        offset += 1;
    }
}

fn write_factor(output_bitvec: &mut BitVec, huffman_table: &HashMap<u8, (u16, u16)>, factor: i32, following_zeros: u8) {
    let (factor, non_zero_digits) = if factor < 0 {
        encode_negative(factor)
    } else {
        (factor, count_non_zero_digits(factor))
    };

    let value = non_zero_digits | (following_zeros << 4);

    write_huffman_code(output_bitvec, huffman_table[&value]);
    if non_zero_digits > 0 {
        write_number_bits(output_bitvec, factor, non_zero_digits);
    }
}

fn write_huffman_code(output_bitvec: &mut BitVec, code: (u16, u16)) {
    write_number_bits(output_bitvec, code.0 as i32, code.1 as u8)
}

fn write_number_bits(output_bitvec: &mut BitVec, number: i32, total_bits: u8) {
    for i in 0..total_bits {
        let index = total_bits - i - 1;
        output_bitvec.push(((number >> index) & 0b1) == 1);
    }
}

fn count_non_zero_digits(value: i32) -> u8 {
    let mut result = 0;

    for i in 0..32 {
        if (value >> i) & 1 == 1 {
            result = i + 1;
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
    let mut prev_dc = [0i32; 3];
    let mut offset = 0;

    for _ in 0..total_mcus {
        for channel_id in 1..=(total_channels as ChannelID) {
            let channel = &channels[&channel_id];

            let dc_huffman_table = &huffman_tables_by_channel[&(HuffmanTableType::DC, channel_id)];
            let ac_huffman_table = &huffman_tables_by_channel[&(HuffmanTableType::AC, channel_id)];

            for _ in 0..channel.vertical_sampling {
                for _ in 0..channel.horizontal_sampling {
                    let (matrix, new_offset) = read_huffman_encoded_matrix(
                        &bitvec, 
                        offset, 
                        &mut prev_dc, 
                        channel_id, 
                        &dc_huffman_table, 
                        &ac_huffman_table
                    );

                    result.push(matrix);
                    offset = new_offset;
                }
            }
        }
    }

    Ok(result)
}

fn read_huffman_encoded_matrix(
    bitvec: &BitVec, 
    offset: usize, 
    prev_dc: &mut [i32; 3], 
    channel_id: ChannelID,
    dc_huffman_table: &HuffmanTable,
    ac_huffman_table: &HuffmanTable
) -> ([i32; 64], usize) {
    let dc_huffman_table = &dc_huffman_table.table;
    let ac_huffman_table = &ac_huffman_table.table;

    let mut offset = offset;
    
    let mut bitgroup = 0;
    let mut bitgroup_length = 0;

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
                        factor = decode_negative(factor, factor_length);
                    }

                    factor_vals[factor_offset] = factor;
                    factor_offset += 1;
                }
            }
        }

        if factor_offset == 64 {
            break;
        }
    }

    factor_vals[0] += prev_dc[channel_id as usize - 1];
    prev_dc[channel_id as usize - 1] = factor_vals[0];
    let matrix = unzigzag_64(&factor_vals);
    //println!("read matrix: {:?}", matrix);

    (matrix, offset)
}

fn encode_negative(number: i32) -> (i32, u8) {
    let mut mask = 0;
    let mut len = 0;
    for _ in 0..(count_non_zero_digits(-number)) {
        mask = (mask << 1) | 1;
        len += 1;
    }

    let result = (-number) ^ mask;
    (result, len)
}

fn decode_negative(number: i32, code_length: u8) -> i32 {
    number - 2i32.pow(code_length as u32) + 1
}

pub fn unescape_image_data(data: &[u8]) -> Result<(Vec<u8>, usize), JPEGReaderError> {
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

    Ok((new_data, offset))
}

pub fn escape_image_data(data: &[u8]) -> Vec<u8> {
    let mut new_data = Vec::with_capacity(data.len());

    for element in data {
        new_data.push(*element);

        if *element == 0xFF {
            new_data.push(0x00);
        }
    }

    new_data
}

#[cfg(test)]
mod tests {
    
    use super::*;
    use crate::writer::{AC_HUFFAN_Y, DC_HUFFMAN_Y};

    #[test]
    fn test_black_to_ycbcr() {
        let ycbcr = rgb_to_ycbcr(&Pixel::from_rgb(0, 0, 0));
        let rgb = ycbcr_to_rgb(ycbcr.0, ycbcr.1, ycbcr.2);

        assert_eq!(rgb, (0, 0, 0));
    }

    #[test]
    fn test_count_nonzero_digits_5() {
        assert_eq!(count_non_zero_digits(5), 3);
    }

    #[test]
    fn test_count_nonzero_digits_96() {
        assert_eq!(count_non_zero_digits(96), 7);
    }

    #[test]
    fn test_encode_negative() {
        for i in 1..250 {
            let encoded = encode_negative(-i);
            let decoded = decode_negative(encoded.0, encoded.1);
        
            assert_eq!(decoded, -i);
        }
    }

    #[test]
    fn test_encode_negative_8() {
        assert_eq!(encode_negative(-8), (7, 4));
    }

    #[test]
    fn test_escape_unescape() {
        let test_data_escaped: Vec<u8> = vec![
            246, 11, 123, 1, 227, 175, 218, 14, 63, 138, 94, 28, 241, 106, 191, 130, 68, 66, 55, 0, 112, 
            120, 193, 175, 167, 173, 159, 209, 142, 18, 52, 20, 147, 178, 50, 193, 113, 164, 32, 220, 42, 
            47, 117, 232, 124, 111, 227, 169, 12, 190, 44, 241, 159, 138, 124, 46, 119, 68, 164, 144, 167, 
            160, 207, 90, 252, 123, 49, 138, 196, 85, 230, 93, 79, 219, 114, 223, 22, 176, 216, 120, 90, 
            86, 62, 254, 240, 55, 131, 109, 254, 13, 248, 147, 254, 17, 127, 9, 120, 91, 117, 188, 139, 
            187, 204, 13, 142, 61, 115, 95, 140, 229, 156, 75, 136, 171, 89, 70, 79, 83, 241, 76, 95, 8, 
            66, 20, 190, 177, 69, 221, 31, 26, 124, 101, 240, 71, 133, 19, 227, 23, 140, 60, 39, 225, 117, 
            27, 215, 4, 168, 232, 15, 113, 95, 180, 229, 209, 88, 168, 93, 245, 63, 27, 204, 176, 21, 176, 
            117, 189, 199, 162, 122, 31
        ];
    
        let (unescaped, _) = unescape_image_data(&test_data_escaped).expect("Failed to unescape test image data");

        let escaped = escape_image_data(&unescaped);

        assert_eq!(escaped, test_data_escaped);
    }

    #[test]
    fn test_encode_matrix() {
        let matrix = [
            -250, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ];
    
        let mut encoded = BitVec::new();
        let mut prev_dc = [0i32; 3];
        let dc_huffman_table = HuffmanTable::from_vk(0, HuffmanTableType::DC, &DC_HUFFMAN_Y);
        let ac_huffman_table = HuffmanTable::from_vk(0, HuffmanTableType::AC, &AC_HUFFAN_Y);

        write_huffman_encoded_matrix(
            &mut encoded, 
            &mut prev_dc, 
            &dc_huffman_table, 
            &ac_huffman_table, 
            1, 
            &matrix
        );

        let mut prev_dc = [0i32; 3];

        let (matrix_decoded, _) = read_huffman_encoded_matrix(
            &encoded, 
            0, 
            &mut prev_dc, 
            1, 
            &dc_huffman_table, 
            &ac_huffman_table
        );

        assert_eq!(matrix_decoded, matrix);
    }
}