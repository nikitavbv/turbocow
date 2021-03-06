use std::f32::consts::PI;

use lazy_static::lazy_static;
use maplit::hashmap;

use core::models::{image::Image, pixel::Pixel, io::{ImageIOError, ImageWriter, ImageWriterOptions}};
use std::{collections::HashMap};

use byteorder::{BigEndian, ByteOrder};

use crate::{common::{Channel, HuffmanTable, HuffmanTableType}, common::{ChannelID, rgb_to_ycbcr}, common::write_huffman_encoded_channels_data, common::zigzag, common::escape_image_data};

const OPTION_QUALITY: &'static str = "quality";

// These tables are used by GIMP when saving with 90% quality.
const QUANTIZATION_TABLE_Y_90: [i32; 64] = [
     3,  2,  2,  3,  5,  8, 10, 12, 
     2,  2,  3,  4,  5, 12, 12, 11, 
     3,  3,  3,  5,  8, 11, 14, 11, 
     3,  3,  4,  6, 10, 17, 16, 12, 
     4,  4,  7, 11, 14, 22, 21, 15, 
     5,  7, 11, 13, 16, 21, 23, 18, 
    10, 13, 16, 17, 21, 24, 24, 20, 
    14, 18, 19, 20, 22, 20, 21, 20
];

const QUANTIZATION_TABLE_CB_CR_90: [i32; 64] = [
     3,  4,  5,  9, 20, 20, 20, 20, 
     4,  4,  5, 13, 20, 20, 20, 20, 
     5,  5, 11, 20, 20, 20, 20, 20, 
     9, 13, 20, 20, 20, 20, 20, 20, 
    20, 20, 20, 20, 20, 20, 20, 20, 
    20, 20, 20, 20, 20, 20, 20, 20, 
    20, 20, 20, 20, 20, 20, 20, 20, 
    20, 20, 20, 20, 20, 20, 20, 20
];

// 100% quality
const QUANTIZATION_TABLE_Y_100: [i32; 64] = [
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1
];

const QUANTIZATION_TABLE_CB_CR_100: [i32; 64] = [
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1, 
    1, 1, 1, 1, 1, 1, 1, 1
];

// 50% quality
const QUANTIZATION_TABLE_Y_50: [i32; 64] = [
    16, 11, 10, 16,  24,  40,  51,  61, 
    12, 12, 14, 19,  26,  58,  60,  55, 
    14, 13, 16, 24,  40,  57,  69,  56, 
    14, 17, 22, 29,  51,  87,  80,  62, 
    18, 22, 37, 56,  68, 109, 103,  77, 
    24, 35, 55, 64,  81, 104, 113,  92, 
    49, 64, 78, 87, 103, 121, 120, 101, 
    72, 92, 95, 98, 112, 100, 103,  99
];

const QUANTIZATION_TABLE_CB_CR_50: [i32; 64] = [
    17, 18, 24, 47, 99, 99, 99, 99, 
    18, 21, 26, 66, 99, 99, 99, 99, 
    24, 26, 56, 99, 99, 99, 99, 99,
    47, 66, 99, 99, 99, 99, 99, 99, 
    99, 99, 99, 99, 99, 99, 99, 99, 
    99, 99, 99, 99, 99, 99, 99, 99, 
    99, 99, 99, 99, 99, 99, 99, 99, 
    99, 99, 99, 99, 99, 99, 99, 99
];

// 25% quality
const QUANTIZATION_TABLE_Y_25: [i32; 64] = [
     32,  22,  20,  32,  48,  80, 102, 122, 
     24,  24,  28,  38,  52, 116, 120, 110, 
     28,  26,  32,  48,  80, 114, 138, 112, 
     28,  34,  44,  58, 102, 174, 160, 124, 
     36,  44,  74, 112, 136, 218, 206, 154, 
     48,  70, 110, 128, 162, 208, 226, 184, 
     98, 128, 156, 174, 206, 242, 240, 202, 
    144, 184, 190, 196, 224, 200, 206, 198
];

const QUANTIZATION_TABLE_CB_CR_25: [i32; 64] = [
     34,  36,  48,  94, 198, 198, 198, 198, 
     36,  42,  52, 132, 198, 198, 198, 198, 
     48,  52, 112, 198, 198, 198, 198, 198, 
     94, 132, 198, 198, 198, 198, 198, 198, 
    198, 198, 198, 198, 198, 198, 198, 198, 
    198, 198, 198, 198, 198, 198, 198, 198, 
    198, 198, 198, 198, 198, 198, 198, 198, 
    198, 198, 198, 198, 198, 198, 198, 198
];

// it turns out that majority of jpeg encoders use pre-defined Huffman tables from standard, instead of
// generating own tables. This approach actually provides good enough approximation. In this encoding
// we will use the same approach. These tables are copied from GIMP when exporting at 90% quality.
// Tables represented as value => (code, code_length).
lazy_static! {
    pub static ref DC_HUFFMAN_Y: HashMap<u8, (u16, u16)> = hashmap!{
        11 => (510, 9),
        3 => (4, 3),
        9 => (126, 7),
        8 => (62, 6),
        6 => (14, 4),
        5 => (6, 3),
        0 => (0, 2),
        4 => (5, 3),
        7 => (30, 5),
        2 => (3, 3),
        1 => (2, 3),
        10 => (254, 8),        
    };

    static ref DC_HUFFMAN_CBCR: HashMap<u8, (u16, u16)> = hashmap!{
        0 => (0, 2),
        5 => (30, 5),
        11 => (2046, 11),
        10 => (1022, 10),
        3 => (6, 3),
        7 => (126, 7),
        8 => (254, 8),
        2 => (2, 2),
        1 => (1, 2),
        9 => (510, 9),
        6 => (62, 6),
        4 => (14, 4),
    };

    pub static ref AC_HUFFAN_Y: HashMap<u8, (u16, u16)> = hashmap!{
        170 => (65487, 16),
        118 => (65457, 16),
        69 => (65432, 16),
        149 => (65473, 16),
        210 => (65506, 16),
        90 => (65445, 16),
        17 => (12, 4),
        22 => (65412, 16),
        202 => (65505, 16),
        72 => (65435, 16),
        5 => (26, 5),
        81 => (122, 7),
        152 => (65476, 16),
        245 => (65529, 16),
        88 => (65443, 16),
        195 => (65498, 16),
        165 => (65482, 16),
        25 => (65415, 16),
        201 => (65504, 16),
        183 => (65493, 16),
        34 => (249, 8),
        146 => (65470, 16),
        33 => (28, 5),
        154 => (65478, 16),
        24 => (65414, 16),
        217 => (65513, 16),
        147 => (65471, 16),
        250 => (65534, 16),
        66 => (1016, 10),
        214 => (65510, 16),
        49 => (58, 6),
        248 => (65532, 16),
        178 => (65488, 16),
        101 => (65448, 16),
        8 => (1014, 10),
        71 => (65434, 16),
        57 => (65428, 16),
        167 => (65484, 16),
        198 => (65501, 16),
        130 => (32704, 15),
        70 => (65433, 16),
        87 => (65442, 16),
        105 => (65452, 16),
        0 => (10, 4),
        82 => (2039, 11),
        242 => (65526, 16),
        186 => (65496, 16),
        104 => (65451, 16),
        113 => (250, 8),
        67 => (65430, 16),
        218 => (65514, 16),
        103 => (65450, 16),
        3 => (4, 3),
        233 => (65523, 16),
        19 => (121, 7),
        21 => (2038, 11),
        182 => (65492, 16),
        39 => (65419, 16),
        249 => (65533, 16),
        51 => (4085, 12),
        197 => (65500, 16),
        212 => (65508, 16),
        179 => (65489, 16),
        7 => (248, 8),
        181 => (65491, 16),
        246 => (65530, 16),
        180 => (65490, 16),
        243 => (65527, 16),
        84 => (65439, 16),
        247 => (65531, 16),
        145 => (505, 9),
        169 => (65486, 16),
        54 => (65425, 16),
        229 => (65519, 16),
        135 => (65466, 16),
        50 => (503, 9),
        38 => (65418, 16),
        228 => (65518, 16),
        37 => (65417, 16),
        10 => (65411, 16),
        196 => (65499, 16),
        1 => (0, 2),
        41 => (65421, 16),
        164 => (65481, 16),
        215 => (65511, 16),
        119 => (65458, 16),
        244 => (65528, 16),
        177 => (1017, 10),
        226 => (65516, 16),
        193 => (1018, 10),
        163 => (65480, 16),
        98 => (4086, 12),
        241 => (65525, 16),
        209 => (2040, 11),
        137 => (65468, 16),
        83 => (65438, 16),
        134 => (65465, 16),
        211 => (65507, 16),
        150 => (65474, 16),
        73 => (65436, 16),
        122 => (65461, 16),
        35 => (1015, 10),
        120 => (65459, 16),
        2 => (1, 2),
        162 => (65479, 16),
        9 => (65410, 16),
        6 => (120, 7),
        4 => (11, 4),
        216 => (65512, 16),
        36 => (4084, 12),
        129 => (504, 9),
        89 => (65444, 16),
        234 => (65524, 16),
        114 => (4087, 12),
        97 => (123, 7),
        86 => (65441, 16),
        23 => (65413, 16),
        106 => (65453, 16),
        225 => (65515, 16),
        52 => (65423, 16),
        55 => (65426, 16),
        115 => (65454, 16),
        26 => (65416, 16),
        161 => (506, 9),
        65 => (59, 6),
        99 => (65446, 16),
        133 => (65464, 16),
        132 => (65463, 16),
        85 => (65440, 16),
        56 => (65427, 16),
        213 => (65509, 16),
        116 => (65455, 16),
        230 => (65520, 16),
        58 => (65429, 16),
        74 => (65437, 16),
        166 => (65483, 16),
        68 => (65431, 16),
        232 => (65522, 16),
        168 => (65485, 16),
        131 => (65462, 16),
        40 => (65420, 16),
        227 => (65517, 16),
        199 => (65502, 16),
        100 => (65447, 16),
        121 => (65460, 16),
        138 => (65469, 16),
        185 => (65495, 16),
        151 => (65475, 16),
        148 => (65472, 16),
        153 => (65477, 16),
        20 => (502, 9),
        42 => (65422, 16),
        102 => (65449, 16),
        194 => (65497, 16),
        240 => (2041, 11),
        184 => (65494, 16),
        117 => (65456, 16),
        53 => (65424, 16),
        231 => (65521, 16),
        136 => (65467, 16),
        200 => (65503, 16),
        18 => (27, 5),    
    };

    static ref AC_HUFFAN_CBCR: HashMap<u8, (u16, u16)> = hashmap! {
        165 => (65484, 16),
        212 => (65510, 16),
        245 => (65529, 16),
        154 => (65480, 16),
        166 => (65485, 16),
        121 => (65461, 16),
        215 => (65513, 16),
        67 => (65431, 16),
        240 => (1018, 10),
        132 => (65465, 16),
        3 => (10, 4),
        117 => (65457, 16),
        83 => (65439, 16),
        65 => (58, 6),
        36 => (4086, 12),
        169 => (65488, 16),
        55 => (65427, 16),
        137 => (65470, 16),
        21 => (2038, 11),
        52 => (4087, 12),
        74 => (65438, 16),
        39 => (65421, 16),
        145 => (503, 9),
        185 => (65497, 16),
        150 => (65476, 16),
        178 => (65490, 16),
        25 => (65418, 16),
        180 => (65492, 16),
        152 => (65478, 16),
        133 => (65466, 16),
        138 => (65471, 16),
        201 => (65506, 16),
        241 => (32707, 15),
        194 => (65499, 16),
        244 => (65528, 16),
        211 => (65509, 16),
        5 => (25, 5),
        42 => (65424, 16),
        51 => (1016, 10),
        105 => (65453, 16),
        38 => (65420, 16),
        162 => (65481, 16),
        19 => (246, 8),
        168 => (65487, 16),
        69 => (65433, 16),
        106 => (65454, 16),
        57 => (65429, 16),
        196 => (65501, 16),
        242 => (65526, 16),
        101 => (65449, 16),
        88 => (65444, 16),
        102 => (65450, 16),
        10 => (4084, 12),
        56 => (65428, 16),
        41 => (65423, 16),
        233 => (65524, 16),
        86 => (65442, 16),
        136 => (65469, 16),
        99 => (65447, 16),
        89 => (65445, 16),
        118 => (65458, 16),
        202 => (65507, 16),
        135 => (65468, 16),
        229 => (65520, 16),
        9 => (1014, 10),
        232 => (65523, 16),
        164 => (65483, 16),
        113 => (122, 7),
        20 => (501, 9),
        151 => (65477, 16),
        90 => (65446, 16),
        6 => (56, 6),
        116 => (65456, 16),
        228 => (65519, 16),
        34 => (247, 8),
        199 => (65504, 16),
        104 => (65452, 16),
        33 => (26, 5),
        4 => (24, 5),
        17 => (11, 4),
        82 => (1017, 10),
        218 => (65516, 16),
        177 => (505, 9),
        129 => (249, 8),
        68 => (65432, 16),
        87 => (65443, 16),
        198 => (65503, 16),
        114 => (2040, 11),
        153 => (65479, 16),
        230 => (65521, 16),
        161 => (504, 9),
        84 => (65440, 16),
        147 => (65473, 16),
        163 => (65482, 16),
        115 => (65455, 16),
        40 => (65422, 16),
        73 => (65437, 16),
        179 => (65491, 16),
        227 => (65518, 16),
        58 => (65430, 16),
        8 => (500, 9),
        183 => (65495, 16),
        37 => (32706, 15),
        130 => (65463, 16),
        26 => (65419, 16),
        170 => (65489, 16),
        234 => (65525, 16),
        250 => (65534, 16),
        146 => (65472, 16),
        181 => (65493, 16),
        7 => (120, 7),
        81 => (59, 6),
        85 => (65441, 16),
        24 => (65417, 16),
        213 => (65511, 16),
        49 => (27, 5),
        2 => (4, 3),
        98 => (2039, 11),
        54 => (65426, 16),
        66 => (502, 9),
        131 => (65464, 16),
        97 => (121, 7),
        247 => (65531, 16),
        193 => (506, 9),
        134 => (65467, 16),
        186 => (65498, 16),
        122 => (65462, 16),
        246 => (65530, 16),
        248 => (65532, 16),
        53 => (65425, 16),
        200 => (65505, 16),
        167 => (65486, 16),
        148 => (65474, 16),
        22 => (4085, 12),
        149 => (65475, 16),
        50 => (248, 8),
        214 => (65512, 16),
        210 => (65508, 16),
        35 => (1015, 10),
        243 => (65527, 16),
        197 => (65502, 16),
        70 => (65434, 16),
        225 => (16352, 14),
        100 => (65448, 16),
        217 => (65515, 16),
        249 => (65533, 16),
        226 => (65517, 16),
        18 => (57, 6),
        23 => (65416, 16),
        119 => (65459, 16),
        182 => (65494, 16),
        71 => (65435, 16),
        120 => (65460, 16),
        195 => (65500, 16),
        0 => (0, 2),
        209 => (2041, 11),
        103 => (65451, 16),
        184 => (65496, 16),
        72 => (65436, 16),
        1 => (1, 2),
        216 => (65514, 16),
        231 => (65522, 16),
    };

    // For DCT
    static ref COS_TABLE: [f32; 64] = precompute_cos_table();
}

pub struct JPEGWriter {
}

impl JPEGWriter {

    pub fn new() -> Self {
        JPEGWriter {
        }
    }
}

impl ImageWriter for JPEGWriter {
    
    fn write(&self, image: &Image, options: &ImageWriterOptions) -> Result<Vec<u8>, ImageIOError> {
        // initialization
        let quality = options.get_u32(OPTION_QUALITY, 90)?;

        let quantization_tables: HashMap<u8, [i32; 64]> = hashmap! {
            0 => match quality {
                90 => QUANTIZATION_TABLE_Y_90,
                100 => QUANTIZATION_TABLE_Y_100,
                50 => QUANTIZATION_TABLE_Y_50,
                25 => QUANTIZATION_TABLE_Y_25,
                other => return Err(ImageIOError::InvalidOptions {
                    description: format!("Quality level is not supported: {}", other),
                })
            }.clone(),
            1 => match quality {
                90 => QUANTIZATION_TABLE_CB_CR_90,
                100 => QUANTIZATION_TABLE_CB_CR_100,
                50 => QUANTIZATION_TABLE_CB_CR_50,
                25 => QUANTIZATION_TABLE_CB_CR_25,
                other => return Err(ImageIOError::InvalidOptions {
                    description: format!("Quality level is not supported: {}", other),
                })
            }.clone(),
        };

        let quantization_table_by_channel: HashMap<u8, u8> = hashmap! {
            1 => 0,
            2 => 1,
            3 => 1,
        };

        let huffman_tables_by_channel: HashMap<(HuffmanTableType, ChannelID), HuffmanTable> = hashmap! {
            (HuffmanTableType::DC, 1) => HuffmanTable::from_vk(0, HuffmanTableType::DC, &DC_HUFFMAN_Y),
            (HuffmanTableType::DC, 2) => HuffmanTable::from_vk(1, HuffmanTableType::DC, &DC_HUFFMAN_CBCR),
            (HuffmanTableType::DC, 3) => HuffmanTable::from_vk(1, HuffmanTableType::DC, &DC_HUFFMAN_CBCR),
            (HuffmanTableType::AC, 1) => HuffmanTable::from_vk(0, HuffmanTableType::AC, &AC_HUFFAN_Y),
            (HuffmanTableType::AC, 2) => HuffmanTable::from_vk(1, HuffmanTableType::AC, &AC_HUFFAN_CBCR),
            (HuffmanTableType::AC, 3) => HuffmanTable::from_vk(1, HuffmanTableType::AC, &AC_HUFFAN_CBCR),
        };

        // given image, transform it into multiple blocks 8x8.
        let mcus: Vec<[Pixel; 64]> = image_to_mcus(&image);
        trace!("total mcus: {}", mcus.len());

        let mut matrices: Vec<[[i32; 64]; 3]> = Vec::new();
        for mcu in mcus {
            let mut mcu_pixels = [[0i32; 3]; 64];

            for i in 0..mcu_pixels.len() {
                let ycbcr = rgb_to_ycbcr(&mcu[i]);
                mcu_pixels[i] = [ycbcr.0, ycbcr.1, ycbcr.2];
            }

            let mut mcu_matrices = [[0i32; 64]; 3];

            for channel in 0..mcu_pixels[0].len() {
                let channel_id = channel + 1;
                let quantization_table_id = quantization_table_by_channel[&(channel_id as u8)];
                let quantization_table = quantization_tables[&quantization_table_id];

                let channel = extract_channel(&mcu_pixels, channel);
                let channel = dct_encode(&channel);
                let channel = divide_64s(&channel, &quantization_table);
                
                mcu_matrices[channel_id - 1] = channel;
            }

            matrices.push(mcu_matrices);
        }

        let image_data = write_huffman_encoded_channels_data(
            &matrices,
            &huffman_tables_by_channel
        );
        let image_data_encoded = escape_image_data(&image_data);

        let channels: Vec<Channel> = (1..=3).map(|id| {
            Channel {
                id,
                horizontal_sampling: 1,
                vertical_sampling: 1,
                quantization_table_id: *quantization_table_by_channel.get(&id)
                    .expect("Expected quantization table to be present, because quatization tables were statically defined for 3 channels"),
            }
        }).collect();

        // writing
        let mut data = vec![0xFF, 0xD8]; // start with magic
        // writing quantization tables 
        for (table_id, table) in quantization_tables {
            data.append(&mut prepend_marker(0xDB, write_quantization_table(table_id, &table)));
        }
        // writing baseline dct
        data.append(&mut prepend_marker(0xC0, write_baseline_dct(
            image.width as u16, 
            image.height as u16,
            &channels
        )));
        // writing huffman tables
        data.append(&mut prepend_marker(0xC4, write_huffman_table(HuffmanTableType::DC, 0)));
        data.append(&mut prepend_marker(0xC4, write_huffman_table(HuffmanTableType::DC, 1)));
        data.append(&mut prepend_marker(0xC4, write_huffman_table(HuffmanTableType::AC, 0)));
        data.append(&mut prepend_marker(0xC4, write_huffman_table(HuffmanTableType::AC, 1)));

        // start of scan
        data.append(&mut prepend_marker(0xDA, write_start_of_scan(image_data_encoded)));

        // end of data
        data.append(&mut prepend_marker(0xD9, Vec::new()));

        Ok(data)
    }
}

fn write_start_of_scan(data: Vec<u8>) -> Vec<u8> {
    let mut flat_data = data;

    let mut data = Vec::new();
    // block length
    data.push(0);
    data.push(12);

    data.push(3); // total channels
    
    // y
    data.push(1); // channel id
    data.push(0); // huffman tables ids

    // cb
    data.push(2); // channel id
    data.push(1 << 4 | 1); // huffman tables ids

    // cr
    data.push(3); // channel id
    data.push(1 << 4 | 1); // huffman tables ids

    data.push(0); // start of spectral or predictor selection
    data.push(0); // end of spectral selection
    data.push(0); // successive approximation bhit position

    data.append(&mut flat_data);

    data
}

fn write_huffman_table(table_type: HuffmanTableType, id: u8) -> Vec<u8> {
    // pre-encoded for static tables we use
    match (table_type, id) {
        (HuffmanTableType::DC, 0) => vec![0, 31, 0, 0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        (HuffmanTableType::AC, 0) => vec![0, 181, 16, 0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 125, 1, 2, 3, 0, 4, 17, 5, 18, 33, 49, 65, 6, 19, 81, 97, 7, 34, 113, 20, 50, 129, 145, 161, 8, 35, 66, 177, 193, 21, 82, 209, 240, 36, 51, 98, 114, 130, 9, 10, 22, 23, 24, 25, 26, 37, 38, 39, 40, 41, 42, 52, 53, 54, 55, 56, 57, 58, 67, 68, 69, 70, 71, 72, 73, 74, 83, 84, 85, 86, 87, 88, 89, 90, 99, 100, 101, 102, 103, 104, 105, 106, 115, 116, 117, 118, 119, 120, 121, 122, 131, 132, 133, 134, 135, 136, 137, 138, 146, 147, 148, 149, 150, 151, 152, 153, 154, 162, 163, 164, 165, 166, 167, 168, 169, 170, 178, 179, 180, 181, 182, 183, 184, 185, 186, 194, 195, 196, 197, 198, 199, 200, 201, 202, 210, 211, 212, 213, 214, 215, 216, 217, 218, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250],
        (HuffmanTableType::DC, 1) => vec![0, 31, 1, 0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        (HuffmanTableType::AC, 1) => vec![0, 181, 17, 0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 119, 0, 1, 2, 3, 17, 4, 5, 33, 49, 6, 18, 65, 81, 7, 97, 113, 19, 34, 50, 129, 8, 20, 66, 145, 161, 177, 193, 9, 35, 51, 82, 240, 21, 98, 114, 209, 10, 22, 36, 52, 225, 37, 241, 23, 24, 25, 26, 38, 39, 40, 41, 42, 53, 54, 55, 56, 57, 58, 67, 68, 69, 70, 71, 72, 73, 74, 83, 84, 85, 86, 87, 88, 89, 90, 99, 100, 101, 102, 103, 104, 105, 106, 115, 116, 117, 118, 119, 120, 121, 122, 130, 131, 132, 133, 134, 135, 136, 137, 138, 146, 147, 148, 149, 150, 151, 152, 153, 154, 162, 163, 164, 165, 166, 167, 168, 169, 170, 178, 179, 180, 181, 182, 183, 184, 185, 186, 194, 195, 196, 197, 198, 199, 200, 201, 202, 210, 211, 212, 213, 214, 215, 216, 217, 218, 226, 227, 228, 229, 230, 231, 232, 233, 234, 242, 243, 244, 245, 246, 247, 248, 249, 250],
        other => panic!("Unexpected Huffman table to write: {:?}", other),
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

fn divide_64s(a: &[i32; 64], b: &[i32; 64]) -> [i32; 64] {
    let mut result = [0i32; 64];

    for i in 0..64 {
        result[i] = a[i] / b[i];
    }

    result
}

fn dct_encode(values: &[i32; 64]) -> [i32; 64] {
    let mut new_values = [0i32; 64];
    for i in 0..64 {
        new_values[i] = values[i] - 128;
    }
    let values = new_values;

    let mut result = [0i32; 64];

    for u in 0..8 {
        for v in 0..8 {
            let mut sum = 0 as f32;

            for y in 0..8 {
                for x in 0..8 {
                    sum += (values[y * 8 + x] as f32) * COS_TABLE[y * 8 + u] * COS_TABLE[x * 8 + v];
                }
            }

            let cu = if u == 0 { 1.0/2f32.sqrt() } else { 1.0 };
            let cv = if v == 0 { 1.0/2f32.sqrt() } else { 1.0 };
            result[u * 8 + v] = (sum * cu * cv / 4.0).round() as i32;
        }
    }

    result
}

fn precompute_cos_table() -> [f32; 64] {
    let mut result = [0f32; 64];

    for i in 0..8 {
        for j in 0..8 {
            result[i * 8 + j] = (((2 * i + 1) * j) as f32 * PI / 16.0).cos()
        }
    }

    result
}

fn extract_channel(pixels: &[[i32; 3]; 64], channel_index: usize) -> [i32; 64] {
    let mut channel = [0i32; 64];

    for i in 0..pixels.len() {
        channel[i] = pixels[i][channel_index];
    }

    channel
}

fn image_to_mcus(image: &Image) -> Vec<[Pixel; 64]> {
    let mut mcus = Vec::new();

    for y in 0..(image.height as f32 / 8.0).ceil() as usize {
        for x in 0..(image.width as f32 / 8.0).ceil() as usize {
            mcus.push(image_mcu(&image, y, x));
        }
    }

    mcus
}

fn image_mcu(image: &Image, y: usize, x: usize) -> [Pixel; 64] {
    let mut result = [Pixel::black(); 64];
    
    let offset_x = x * 8;
    let offset_y = y * 8;

    for y in 0..8 {
        for x in 0..8 {
            result[y * 8 + x] = if offset_x + x < image.width && offset_y + y < image.height {
                image.get_pixel(offset_x + x, offset_y + y)
            } else  {
                Pixel::black()
            };
        }
    }

    result
}

#[cfg(test)]
mod tests {

    use super::*;

    use std::fs::read;

    use bit_vec::BitVec;

    use core::models::io::{ImageReader, ImageWriterOptions};

    use crate::{common::ChannelID, common::rgb_to_ycbcr, common::{read_huffman_encoded_channels_data, ycbcr_to_rgb}, reader::{JPEGReader, dct_decode}};

    #[test]
    fn test_rgb_to_ycbcr() {
        let pixel = Pixel::from_rgb(20, 42, 253);
        let ycbcr = rgb_to_ycbcr(&pixel);
        let rgb = ycbcr_to_rgb(ycbcr.0, ycbcr.1, ycbcr.2);

        assert!((20 - rgb.0 as i32).abs() < 5);
        assert!((42 - rgb.1 as i32).abs() < 5);
        assert!((253 - rgb.2 as i32).abs() < 5);
    }

    #[test]
    fn test_write_simple() {
        let image_data = read("assets/balloon.jpg")
            .expect("failed to load test image");
        
        let reader = JPEGReader::new();
        let images = reader.read(&image_data)
            .expect("failed to read test image");
        let image = &images[0];

        let writer = JPEGWriter::new();
        let new_image_data = writer.write(&image, &ImageWriterOptions::default())
            .expect("failed to write image");

        let new_images = reader.read(&new_image_data)
            .expect("failed to read new image");
        let new_image = &new_images[0];

        assert_eq!(new_image.get_pixel(277, 276), Pixel::from_rgb(24, 28, 40));
        assert_eq!(new_image.get_pixel(833, 386), Pixel::from_rgb(150, 28, 43));
    }

    #[test]
    fn test_dct_encode() {
        let source = [
            78, 76, 81, 83, 78, 79, 82, 79, 77, 76, 80, 82, 78, 79, 82, 81, 79, 78, 81, 82, 79, 80, 
            82, 82, 81, 81, 82, 82, 80, 81, 82, 82, 80, 81, 81, 80, 81, 81, 82, 83, 77, 80, 79, 79, 
            82, 82, 81, 83, 78, 81, 80, 79, 83, 82, 80, 82, 81, 84, 82, 81, 84, 82, 78, 80
        ];

        let encoded = dct_encode(&source);        
        let decoded = dct_decode(&encoded);

        let mut diff = 0;
        for i in 0..64 {
            diff += (source[i] - decoded[i]).abs();
        }

        assert!(diff < 50);
    }

    #[test]
    fn test_dct_encode_simple_block() {
        let source = [
            234, 212, 153, 111, 110, 153, 209, 224, 235, 207, 134, 79, 73, 119, 188, 216, 238, 206, 121, 
            50, 39, 86, 162, 207, 248, 216, 133, 64, 53, 95, 164, 214, 248, 227, 165, 115, 110, 140, 190, 
            230, 207, 201, 179, 162, 167, 184, 205, 224, 121, 135, 160, 182, 197, 201, 194, 190, 49, 78, 
            135, 185, 207, 202, 176, 156
        ];
    
        let expected_encoded = [
            288, -16, 262, -12, -25, -8, -10, 0, -30, 116, 246, -4, -20, 0, -12, 0, -3, -105, -162, 15, 0, 
            11, 0, 0, 135, 9, -4, 6, 0, 0, 0, 0, -8, -4, -7, 0, 0, 0, 0, 0, -5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ];

        let encoded = dct_encode(&source);
        
        let mut diff = 0;
        for i in 0..64 {
            diff += (expected_encoded[i] - encoded[i]).abs();
        }

        assert!(diff < 50);
    }

    #[test]
    fn test_huffman_encode() {
        let encoded_data = [
            246, 11, 123, 1, 227, 175, 218, 14, 63, 138, 94, 28, 241, 106, 191, 130, 68, 66, 55, 0, 112, 120, 193, 
            175, 167, 173, 159, 209, 142, 18, 52, 20, 147, 178, 50, 193, 113, 164, 32, 220, 42, 47, 117, 232, 124, 
            111, 227, 169, 12, 190, 44, 241, 159, 138, 124, 46, 119, 68, 164, 144, 167, 160, 207, 90, 252, 123, 49, 
            138, 196, 85, 230, 93, 79, 219, 114, 223, 22, 176, 216, 120, 90, 86, 62, 254, 240, 55, 131, 109, 254, 
            13, 248, 147, 254, 17, 127, 9, 120, 91, 117, 188, 139, 187, 204, 13, 142, 61, 115, 95, 140, 229, 156, 
            75,  136, 171, 89, 70, 79, 83, 241, 76, 95, 8, 66, 20, 190, 177, 69, 221, 31, 26, 124, 101, 240, 71, 
            133, 19, 227, 23, 140, 60, 39, 225, 117, 27, 215, 4, 168, 232, 15, 113, 95, 180, 229, 209, 88, 168, 93, 
            245, 63, 27, 204, 176, 21, 176, 117, 189, 199, 162, 122, 31
        ];
    
        let total_mcus = 4;

        let channels = hashmap! {
            1 => Channel {
                id: 1,
                horizontal_sampling: 1,
                vertical_sampling: 1,
                quantization_table_id: 0,
            },
            2 => Channel {
                id: 2,
                horizontal_sampling: 1,
                vertical_sampling: 1,
                quantization_table_id: 0,
            },
            3 => Channel {
                id: 3,
                horizontal_sampling: 1,
                vertical_sampling: 1,
                quantization_table_id: 0,
            }
        };

        let huffman_tables = hashmap! {
            (HuffmanTableType::DC, 1 as ChannelID) => HuffmanTable::from_vk(0, HuffmanTableType::DC, &DC_HUFFMAN_Y),
            (HuffmanTableType::DC, 2 as ChannelID) => HuffmanTable::from_vk(1, HuffmanTableType::DC, &DC_HUFFMAN_CBCR),
            (HuffmanTableType::DC, 3 as ChannelID) => HuffmanTable::from_vk(1, HuffmanTableType::DC, &DC_HUFFMAN_CBCR),
            (HuffmanTableType::AC, 1 as ChannelID) => HuffmanTable::from_vk(0, HuffmanTableType::AC, &AC_HUFFAN_Y),
            (HuffmanTableType::AC, 2 as ChannelID) => HuffmanTable::from_vk(1, HuffmanTableType::AC, &AC_HUFFAN_CBCR),
            (HuffmanTableType::AC, 3 as ChannelID) => HuffmanTable::from_vk(1, HuffmanTableType::AC, &AC_HUFFAN_CBCR),
        };

        let mut matrices = read_huffman_encoded_channels_data(
            &BitVec::from_bytes(&encoded_data),
            total_mcus,
            &channels,
            &huffman_tables
        ).expect("failed to read test huffman encoded data");

        let mut matrices_transformed: Vec<[[i32; 64]; 3]> = Vec::new();
        while matrices.len() > 0 {
            matrices_transformed.push([
                matrices.remove(0),
                matrices.remove(0),
                matrices.remove(0)
            ]);
        }

        let encoded = write_huffman_encoded_channels_data(&matrices_transformed, &huffman_tables);
        assert_eq!(encoded, encoded_data);
    }
}