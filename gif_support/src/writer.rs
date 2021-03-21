use core::models::{image::Image, pixel::Pixel, io::{ImageIOError, ImageWriter, ImageWriterOptions}};
use std::{time::Duration, cmp::min};

use bit_vec::BitVec;
use byteorder::{ByteOrder, LittleEndian};

use crate::{clustering::{cluster, reduce_colors}, common::{ColorTable, init_dictionary, should_increase_code_size}};

pub const OPTION_MAX_COLORS: &str = "max_colors";

pub struct GIFWriter {
}

impl GIFWriter {

    pub fn new() -> Self {
        GIFWriter {
        }
    }
}

impl ImageWriter for GIFWriter {
    
    fn write(&self, image: &Image, options: &ImageWriterOptions) -> Result<Vec<u8>, ImageIOError> {
        let (image, global_color_table) = adjust_image_with_options(&image, &options)?;

        let mut data = vec![];
        let header = write_header(&image, &global_color_table)?;
        let color_table_data = write_color_table(&global_color_table);
        let local_image_descriptor = write_local_image_descriptor(&image)?;
        let image_data = write_image_data(&image, &global_color_table)?;

        data.append(&mut header.clone());
        data.append(&mut color_table_data.clone());
        data.append(&mut local_image_descriptor.clone());
        data.append(&mut image_data.clone());

        Ok(data)
    }
}

fn write_local_image_descriptor(image: &Image) -> Result<Vec<u8>, ImageIOError> {
    let mut data: Vec<u8> = vec![0 as u8; 10];
    data[0] = 0x2C;

    LittleEndian::write_u16(&mut data[1..3], 0); // left
    LittleEndian::write_u16(&mut data[3..5], 0); // top
    LittleEndian::write_u16(&mut data[5..7], image.width as u16);
    LittleEndian::write_u16(&mut data[7..9], image.height as u16);

    let packed = 0; // no local color table, so 0
    data[9] = packed;

    Ok(data)
}

fn write_header(image: &Image, global_color_table: &ColorTable) -> Result<Vec<u8>, ImageIOError> {
    let mut data: Vec<u8> = vec![0 as u8; 13];

    // GIF
    data[0] = 0x47;
    data[1] = 0x49;
    data[2] = 0x46;

    // 89a
    data[3] = 0x38;
    data[4] = 0x39;
    data[5] = 0x61;

    LittleEndian::write_u16(&mut data[6..8], image.width as u16);
    LittleEndian::write_u16(&mut data[8..10], image.height as u16);

    let mut packed: u8 = 0;
    
    let number_of_global_color_table_entries = global_color_table.colors.len();

    let size_of_global_color_table = (((number_of_global_color_table_entries as f32).log2() as u32) - 1) as u8;
    packed = packed | size_of_global_color_table;
    packed = packed | (6 << 4); // color resolution
    packed = packed | 0b10000000; // use global color table 
    data[10] = packed;

    // data[11] is background color, looks like it is safe to keep 0
    // data[12] is aspect ratio, it if safe to keep 0 here too.

    Ok(data)
}

fn write_color_table(color_table: &ColorTable) -> Vec<u8> {
    let mut data = Vec::new();

    for color in &color_table.colors {
        data.push(color.red);
        data.push(color.green);
        data.push(color.blue);
    }

    data
}

fn write_image_data(image: &Image, color_table: &ColorTable) -> Result<Vec<u8>, ImageIOError> {
    let mut data = vec![];

    let mut pixels: &[Pixel] = &image.pixels;
    let mut bits = BitVec::new();
    let mut dictionary: Vec<Vec<Pixel>> = Vec::new(); // index is a key
    let (mut clear_code, mut end_code) = init_dictionary(&mut dictionary, &color_table);

    let min_code_size = ((dictionary.len() as f64).log2().floor() + 1.0) as u8;
    data.push(min_code_size - 1);

    let mut code_size = min_code_size;

    // add clear code first
    append_bits(&mut bits, clear_code as u16, code_size);

    while image.pixels.len() > 0 {
        let code = find_longest_match(&dictionary, &pixels);
        append_bits(&mut bits, code as u16, code_size);

        pixels = &pixels[dictionary[code].len()..];

        if pixels.len() == 0 {
            break;
        }

        if should_increase_code_size(&dictionary, code_size) {
            code_size += 1;
        }

        let mut new_dictionary_entry = dictionary[code].clone();
        new_dictionary_entry.push(pixels[0].clone());
        dictionary.push(new_dictionary_entry);

        if code_size == 11 {
            append_bits(&mut bits, clear_code as u16, code_size);
            code_size = min_code_size;
            (clear_code, end_code) = init_dictionary(&mut dictionary, &color_table);
        }
    }

    append_bits(&mut bits, end_code as u16, code_size);

    // bits to data
    let compressed_data: Vec<u8> = bits.to_bytes().iter()
        .map(|v| mirror_bits(*v))
        .collect();
    let mut compressed_data: &[u8] = &compressed_data;

    // split compressed data into blocks
    let mut data_blocks: Vec<u8> = Vec::new();

    while compressed_data.len() > 0 {
        let sub_block_size = min(compressed_data.len(), 254);

        data_blocks.push(sub_block_size as u8);
        for i in 0..sub_block_size {
            data_blocks.push(compressed_data[i]);
        }

        compressed_data = &compressed_data[sub_block_size..];
    }

    data_blocks.push(0);
    data_blocks.push(0x3B);

    data.append(&mut data_blocks);

    Ok(data)
}

// 0b10000000 -> 0b00000001
fn mirror_bits(v: u8) -> u8 {
    let mut v = v;
    let mut result = 0;

    for _ in 0..8 {
        result = (result << 1) | (v & 0b1);
        v = v >> 1;
    }

    result
}

fn append_bits(bits: &mut BitVec, code: u16, code_size: u8) {
    for i in 0..code_size {
        bits.push(((code >> i) & 0b1) == 1);
    }
}

fn find_longest_match(dictionary: &Vec<Vec<Pixel>>, pixels: &[Pixel]) -> usize {
    let mut best_match = 0;
    let mut best_match_length = 0;

    // TODO: idea to speed things up: start from the end

    for entry_index in 0..dictionary.len() {
        let entry = &dictionary[entry_index];

        let match_length = match_length(&entry, &pixels);
        if match_length > best_match_length {
            best_match_length = match_length;
            best_match = entry_index;
        }
    }

    best_match
}

fn match_length(a: &[Pixel], b: &[Pixel]) -> usize {
    let mut counter = 0;

    for i in 0..min(a.len(), b.len()) {
        if a[i] == b[i] {
            counter += 1;
        } else {
            break;
        }
    }

    counter
}

fn adjust_image_with_options(image: &Image, options: &ImageWriterOptions) -> Result<(Image, ColorTable), ImageIOError> {
    let max_colors = options.get_u32(OPTION_MAX_COLORS, 256)? as usize;

    info!("reducing colors to {}", max_colors);
    let mut colors = cluster(
        &image.pixels, 
        max_colors, 
        1, 
        10, 
        100, 
        Duration::from_secs(10)
    );

    let target_size = min(2_i32.pow((colors.len() as f32).log2().ceil() as u32), 256) as usize;
    while colors.len() < target_size {
        colors.push(Pixel::black());
    }

    let color_table = ColorTable {
        size: colors.len() * 3,
        colors,
    };
    
    info!("color palette selected, converting image...");
    let image = reduce_colors(&image, &color_table.colors);
    info!("image converted");

    Ok((image, color_table))
}

#[cfg(test)]
mod tests {
    use core::models::{image::Image, io::ImageReader};
    use std::fs::read;

    use crate::reader::{GIFReader, read_image_data};

    use super::*;

    #[test]
    fn test_write_simple() {
        let test_image = Image::test_image();
        let writer = GIFWriter::new();
        let data = writer.write(&test_image, &ImageWriterOptions::default())
            .expect("failed to write test image");

        std::fs::write("assets/result.gif", &data)
            .expect("failed to save the result");
    }

    #[test]
    fn write_test_image_data_simple() {
        let image = Image::test_image();
        let options = ImageWriterOptions::default();
        let (image, global_color_table) = adjust_image_with_options(&image, &options)
            .expect("failed to adjust test image");

        let image_data = write_image_data(&image, &global_color_table)
            .expect("failed to write image data");

        // try reading it
        let image_data_read = read_image_data(&image_data, &global_color_table)
            .expect("failed to read image data");

        assert_eq!(image_data_read.pixels, image.pixels);
    }

    #[test]
    fn write_test_image_data_big() {
        let sunrise = read("assets/sunrise.gif").expect("failed to read test asset");
        
        let reader = GIFReader::new();
        let images = reader.read(&sunrise).expect("failed to read test image");
        let image = &images[0];

        let options = ImageWriterOptions::default();
        let (image, global_color_table) = adjust_image_with_options(&image, &options)
            .expect("failed to adjust test image");

        let image_data = write_image_data(&image, &global_color_table)
            .expect("failed to write image data");

        // try reading it
        let image_data_read = read_image_data(&image_data, &global_color_table)
            .expect("failed to read image data");

        assert_eq!(image_data_read.pixels, image.pixels);
    }

    #[test]
    fn test_write_image_simple() {
        let image = Image::test_image();
        let options = ImageWriterOptions::default();

        let writer = GIFWriter::new();

        let data = writer.write(&image, &options)
            .expect("failed to write test image");

        assert_eq!(data, vec![
            71, 73, 70, 56, 57, 97, 4, 0, 4, 0, 225, 0, 0, 3, 155, 229, 221, 47, 47, 255, 255, 255, 
            0, 0, 0, 44, 0, 0, 0, 0, 4, 0, 4, 0, 0, 2, 5, 148, 13, 128, 113, 86, 0, 59 
        ]);

        // try reading it
        let reader = GIFReader::new();
        let images_read = reader.read(&data)
            .expect("failed to read test image");
        let new_image = &images_read[0];
        
        assert_eq!(new_image.width, image.width);
        assert_eq!(new_image.height, image.height);
        assert_eq!(new_image.pixels, image.pixels);
    }
}
