use byteorder::{LittleEndian, ByteOrder};
use custom_error::custom_error;
use bit_vec::BitVec;

use core::models::{Image, ImageIOError, ImageReader, Pixel};

// see https://www.fileformat.info/format/gif/egff.htm

custom_error! {pub GIFReaderError
    InvalidHeader {description: String} = "Invalid header: {description}",
    InvalidBlock {description: String} = "Invalid block: {description}",
    NotImplemented {description: String} = "Not implemented: {description}"
}

pub struct GIFReader {
}

struct GIF {
    header: Header,
    _global_color_table: GlobalColorTable,
    pixels: Vec<Pixel>,
}

struct Header {

    size: usize, // size of header in bytes

    screen_width: u16,
    screen_height: u16,

    number_of_global_color_table_entries: u32,
}

struct GlobalColorTable {

    size: usize, // size of table in bytes

    colors: Vec<Pixel>,
}

struct CommentExtension {

    size: usize, // size of this block in bytes
}

struct LocalImageDescriptor {

    size: usize, // size of this block in bytes (including separator)
}

impl GIFReader {

    pub fn new() -> Self {
        GIFReader {}
    }
}

impl ImageReader for GIFReader {
    
    fn read(&self, data: &Vec<u8>) -> Result<Image, ImageIOError> {
        let gif = read_gif(&data).map_err(|err| ImageIOError::FailedToRead {
            description: format!("failed to read gif: {}", err)
        })?;

        Ok(Image {
            width: gif.header.screen_width as usize,
            height: gif.header.screen_height as usize,
            pixels: gif.pixels,
        })
    }
}

fn read_gif(data: &Vec<u8>) -> Result<GIF, GIFReaderError> {
    let header = read_header(&data)?;
    let data = &data[header.size..];

    let global_color_table = read_global_color_table(
        &data, 
        header.number_of_global_color_table_entries
    )?;
    let mut data = &data[global_color_table.size..];

    if data[0] == 0x21 && data[1] == 0xFE {
        let block = read_comment_extension(&data)?;
        data = &data[block.size..];
    }

    let local_image_descriptor = read_local_image_descriptor(&data)?;
    let data = &data[local_image_descriptor.size..];

    let pixels = read_image_data(&data, &global_color_table)?;

    Ok(GIF {
        header,
        _global_color_table: global_color_table,
        pixels,
    })
}

fn read_header(data: &Vec<u8>) -> Result<Header, GIFReaderError> {
    match &data[0..3] {
        &[0x47, 0x49, 0x46] => {
            // GIF - ok
        },
        signature => return Err(GIFReaderError::InvalidHeader {
            description: format!("Unexpected signature for GIF: {:x?}", signature),
        })
    };

    match &data[3..6] {
        &[0x38, 0x39, 0x61] => {
            // GIF89a - ok
        },
        version => return Err(GIFReaderError::NotImplemented {
            description: format!("Support for GIF version {:x?} not implemented", version)
        })
    };

    let screen_width = LittleEndian::read_u16(&data[6..8]);
    let screen_height = LittleEndian::read_u16(&data[8..10]);

    let packed: u8 = data[10];

    let size_of_global_color_table = packed & 0b111;
    let color_table_sort_flag = (packed & 0b1000) >> 3;
    let color_resolution = (packed & 0b1110000) >> 4;
    let global_color_table = (packed & 0b10000000) >> 7 == 1;
    let number_of_global_color_table_entries = 1 << (size_of_global_color_table + 1);

    trace!("packed is {:?}", packed);
    trace!("size of global color table: {}", size_of_global_color_table);
    trace!("color table sort flag: {}", color_table_sort_flag);
    trace!("color resolution: {}", color_resolution);
    trace!("global color table: {}", global_color_table);
    trace!("number of global color table entries: {}", number_of_global_color_table_entries);

    if !global_color_table {
        return Err(GIFReaderError::NotImplemented {
            description: "this gif does not use global color table".to_string(),
        });
    }

    let background_color = data[11];
    trace!("background color: {}", background_color);

    let aspect_ratio = data[12];
    trace!("aspect ratio: {}", aspect_ratio);

    Ok(Header {
        size: 13,

        screen_width: screen_width,
        screen_height: screen_height,

        number_of_global_color_table_entries,
    })
}

fn read_global_color_table(data: &[u8], number_of_entries: u32) -> Result<GlobalColorTable, GIFReaderError> {
    let mut colors = Vec::new();

    for i in 0..number_of_entries {
        let offset = (i * 3) as usize;
        colors.push(Pixel::from_rgb(data[offset], data[offset + 1], data[offset + 2]));
    }

    Ok(GlobalColorTable {
        size: (number_of_entries * 3) as usize,
        colors,
    })
}

fn read_comment_extension(data: &[u8]) -> Result<CommentExtension, GIFReaderError> {
    let comment_length = data[2] as usize;
    let block_length = 2 + 1 + comment_length + 1;

    if data[block_length - 1] != 0 {
        return Err(GIFReaderError::InvalidBlock {
            description: format!("unexpected block terminator for comment block: {}", block_length),
        });
    }

    Ok(CommentExtension {
        size: block_length,
    })
}

fn read_local_image_descriptor(data: &[u8]) -> Result<LocalImageDescriptor, GIFReaderError> {
    if data[0] != 0x2C {
        return Err(GIFReaderError::InvalidBlock {
            description: format!("invalid identifier for local image descriptor block: {}", data[0]),
        });
    }

    let _left = LittleEndian::read_u16(&data[1..3]);
    let _top = LittleEndian::read_u16(&data[3..5]);
    let _width = LittleEndian::read_u16(&data[5..7]);
    let _height = LittleEndian::read_u16(&data[7..9]);

    let packed = &data[9];
    if packed & 0b1 == 1 {
        return Err(GIFReaderError::NotImplemented {
            description: "Local color tables are not supported".to_string(),
        });
    }

    Ok(LocalImageDescriptor {
        size: 10,
    })
}

fn read_image_data(data: &[u8], color_table: &GlobalColorTable) -> Result<Vec<Pixel>, GIFReaderError> {
    let min_code_size_in_bits = data[0] + 1;
    let mut data = &data[1..];
    let mut compressed_data: Vec<u8> = Vec::new();

    while data[0] != 0 {
        compressed_data.append(&mut data[1..data[0] as usize+1].to_vec());
        data = &data[data[0] as usize + 1..];
    }

    if data[1] != 0x3B {
        return Err(GIFReaderError::InvalidBlock {
            description: format!("Expected to get 3B at the end of compressed data, got instead: {:x?}", data[1]),
        });
    }

    let data = compressed_data;
    let bits = bit_vec_for_source_bytes(&data);

    let mut pixels: Vec<Pixel> = Vec::new();
    let mut dictionary: Vec<Vec<Pixel>> = Vec::new(); // index is a key
    let (mut clear_index, mut end_index) = init_dictionary(&mut dictionary, &color_table);
    let mut code_size = min_code_size_in_bits;
    let mut offset = 0;
    let mut prev_code = None;

    while offset < bits.len() {
        let code = read_bits(&bits, offset, code_size) as usize;

        if code == clear_index {
            (clear_index, end_index) = init_dictionary(&mut dictionary, &color_table);

            offset += code_size as usize;

            code_size = min_code_size_in_bits;
            prev_code = None;
        } else if code == end_index {
            break;
        } else {
            if code != dictionary.len() {
                let this_code_value = dictionary[code].clone();
                let first_pixel = this_code_value[0].clone();
                pixels.append(&mut this_code_value.clone());

                if let Some(prev_code) = prev_code {
                    let mut prev_code_value = dictionary[prev_code as usize].clone();
                    prev_code_value.push(first_pixel);
                    dictionary.push(prev_code_value);
                }
            } else if let Some(prev_code) = prev_code {
                // match to an entry that has just been encoded.
                let mut prev_code_value = dictionary[prev_code as usize].clone();
                prev_code_value.push(prev_code_value[0]);
                dictionary.push(prev_code_value.clone());
                pixels.append(&mut prev_code_value.clone());
            } else {
                return Err(GIFReaderError::InvalidBlock {
                    description: "Expected prev code to be present when there is a match to entry which has just been encoded".to_string(),
                })
            }

            prev_code = Some(code);
            offset += code_size as usize;
        }

        if dictionary.len() == 2u32.pow(code_size as u32) as usize && code_size < 12 {
            code_size += 1;
        }
    }

    Ok(pixels)
}

fn init_dictionary(dictionary: &mut Vec<Vec<Pixel>>, color_table: &GlobalColorTable) -> (usize, usize) {
    dictionary.clear();
    
    for i in 0..color_table.colors.len() {
        dictionary.push(vec![color_table.colors[i].clone()]);
    }

    let clear_index = dictionary.len();
    dictionary.push(Vec::new());

    let end_index = dictionary.len();
    dictionary.push(Vec::new());

    (clear_index, end_index)
}

fn read_bits(bits: &BitVec, offset: usize, total: u8) -> u16 {
    let mut result = 0;

    for i in 0..total {
        result = result << 1;
        let bit = if bits[offset + (total as usize - 1 - i as usize)] { 1 } else { 0 };
        result = result | bit;
    }
    
    result
}

fn bit_vec_for_source_bytes(data: &[u8]) -> BitVec {
    BitVec::from_fn(data.len() as usize * 8, |x| {
        let res = (data[x / 8] >> (x % 8)) & 0b1 == 1;
        res
    })
}

#[cfg(test)]
mod tests {
    use core::models::Pixel;
    use std::fs::read;

    use super::*;

    #[test]
    fn test_bitvec_offsets() {
        let data: Vec<u8> = vec![
            0b10000000,
            0b00000001,
            0b00000111,
            0b00011100,
        ];

        let bits = bit_vec_for_source_bytes(&data);

        let offset = 0;
        let code_size = 9;
        assert_eq!(read_bits(&bits, offset, code_size), 0b110_000_000);

        let offset = offset + code_size as usize;
        assert_eq!(read_bits(&bits, offset, code_size), 0b110_000_000);

        let offset = offset + code_size as usize;
        let code_size = code_size + 1;
        assert_eq!(read_bits(&bits, offset, code_size), 0b110_000_000_1);
    }

    #[test]
    fn test_read_sunrise() {
        let sunrise = read("assets/sunrise.gif").expect("failed to read test asset");
        
        let reader = GIFReader::new();
        let image = reader.read(&sunrise).expect("failed to read test image");

        assert_eq!(image.width, 1920);
        assert_eq!(image.height, 1279);

        assert_eq!(image.get_pixel(0, 0), Pixel::from_rgb(203, 165, 85));
        assert_eq!(image.get_pixel(1919, 0), Pixel::from_rgb(203, 165, 85));
        assert_eq!(image.get_pixel(1919, 1278), Pixel::from_rgb(0, 130, 153));
        assert_eq!(image.get_pixel(0, 1278), Pixel::from_rgb(0, 130, 153));
    
        assert_eq!(image.get_pixel(1000, 700), Pixel::from_rgb(13, 157, 184));
        assert_eq!(image.get_pixel(1000, 400), Pixel::from_rgb(253, 184, 84));
        assert_eq!(image.get_pixel(600, 250), Pixel::from_rgb(255, 122, 0));
    }

    #[test]
    fn test_read_color_table() {
        let sunrise = read("assets/sunrise.gif").expect("failed to read test asset");

        let gif = read_gif(&sunrise).expect("failed to read test image");
        let global_color_table = gif._global_color_table;

        assert!(global_color_table.colors.contains(&Pixel::from_rgb(255, 122, 0))); // color of the Sun
        assert!(global_color_table.colors.contains(&Pixel::from_rgb(0, 153, 180))); // color of the water
        assert!(global_color_table.colors.contains(&Pixel::from_rgb(69, 65, 65))); // color of the ship
    }
}