use turbocow_core::models::{image::Image, io::{ImageIOError, ImageWriter, ImageWriterOptions}, pixel::Pixel};

use byteorder::{LittleEndian, ByteOrder};

use crate::common::{Compression, DIBHeader, offset_to_far_right};

pub const OPTION_BITS_PER_PIXEL: &str = "bits_per_pixel";
pub const OPTION_USE_ALPHA_CHANNEL: &str = "alpha_channel";

pub const BITFIELDS_16_RED_MASK: u32 = 0b1111100000000000;
pub const BITFIELDS_16_GREEN_MASK: u32 = 0b11111100000;
pub const BITFIELDS_16_BLUE_MASK: u32 = 0b11111;

pub const BITFIELDS_32_RED_MASK: u32 = 0b111111110000000000000000;
pub const BITFIELDS_32_GREEN_MASK: u32 = 0b1111111100000000;
pub const BITFIELDS_32_BLUE_MASK: u32 = 0b11111111;
pub const BITFIELDS_32_ALPHA_MASK: u32 = 0b11111111000000000000000000000000;

pub struct BMPWriter {
}

impl BMPWriter {

    pub fn new() -> Self {
        BMPWriter {
        }
    }
}

impl ImageWriter for BMPWriter {
    
    fn write(&self, image: &Image, options: &ImageWriterOptions) -> Result<Vec<u8>, ImageIOError> {
        let image = adjust_image_with_options(&image, &options)?;

        let dib_header = make_dib_header(&image, &options)?;

        let mut output = vec![];

        let mut dib_header_bytes = write_dib_header(&image, &dib_header)?;
        let dib_header_size = dib_header_bytes.len() as u32;
        output.append(&mut dib_header_bytes);
        
        output.append(&mut write_pixel_array(&image, &dib_header)?);
        
        let mut header = write_header(&output, dib_header_size);
        header.append(&mut output);
        let output = header;

        Ok(output)
    }
}

fn adjust_image_with_options(image: &Image, options: &ImageWriterOptions) -> Result<Image, ImageIOError> {
    let use_alpha_channel = options.get_bool(OPTION_USE_ALPHA_CHANNEL, false)?;
    let mut image: Image = image.clone();
    
    if !use_alpha_channel {
        image = image.compose_alpha_over_background(&Pixel::black());
    }

    Ok(image)
}

fn write_header(data: &Vec<u8>, dib_header_size: u32) -> Vec<u8> {
    let header_size = 14;
    let image_size = header_size + data.len();
    let mut header = vec![0; header_size];

    header[0] = 0x42;
    header[1] = 0x4D;

    LittleEndian::write_u32(&mut header[2..6], image_size as u32);
    LittleEndian::write_u32(&mut header[10..14], dib_header_size + header_size as u32);

    header
}

fn make_dib_header(image: &Image, options: &ImageWriterOptions) -> Result<DIBHeader, ImageIOError> {
    let bit_count = choose_bits_per_pixel(&options)?;
    let bytes_per_pixel = bit_count / 8;

    let compression = choose_compression_method(&options)?;

    let (red_mask, green_mask, blue_mask, alpha_mask) = if compression == Compression::Bitfields {
        if bytes_per_pixel == 2 {
            (BITFIELDS_16_RED_MASK, BITFIELDS_16_GREEN_MASK, BITFIELDS_16_BLUE_MASK, 0)
        } else if bytes_per_pixel == 4 {
            (BITFIELDS_32_RED_MASK, BITFIELDS_32_GREEN_MASK, BITFIELDS_32_BLUE_MASK, BITFIELDS_32_ALPHA_MASK)
        } else {
            return Err(ImageIOError::FailedToWrite {
                description: format!("Mask not set for bytes_per_pixel = {}", bytes_per_pixel),
            });
        }
    } else {
        (0, 0, 0, 0)
    };

    Ok(DIBHeader {
        width: image.width as i32,
        height: image.height as i32,
        bit_count: bit_count as u16,
    
        compression,
    
        red_mask,
        green_mask,
        blue_mask,
        alpha_mask,
    })
}

fn write_dib_header(image: &Image, header_data: &DIBHeader) -> Result<Vec<u8>, ImageIOError> {
    let header_len = 108;
    let mut header = vec![0; header_len];

    let bytes_per_pixel = header_data.bit_count / 8;
    let use_alpha_channel = header_data.alpha_mask != 0;

    LittleEndian::write_u32(&mut header[0..4], header_len as u32);
    LittleEndian::write_i32(&mut header[4..8], image.width as i32);
    LittleEndian::write_i32(&mut header[8..12], image.height as i32);
    LittleEndian::write_u16(&mut header[12..14], 1);
    LittleEndian::write_u16(&mut header[14..16], header_data.bit_count);
    LittleEndian::write_u32(&mut header[16..20], header_data.compression.to_dib_header_value());
    LittleEndian::write_u32(&mut header[20..24], image.width as u32 * image.height as u32 * bytes_per_pixel as u32); // image_size
    LittleEndian::write_i32(&mut header[24..28], 11811); // xpels_per_meter
    LittleEndian::write_i32(&mut header[28..32], 11811); // ypels_per_meter
    
    LittleEndian::write_u32(&mut header[40..44], header_data.red_mask);
    LittleEndian::write_u32(&mut header[44..48], header_data.green_mask);
    LittleEndian::write_u32(&mut header[48..52], header_data.blue_mask);
    if use_alpha_channel {
        LittleEndian::write_u32(&mut header[52..56], BITFIELDS_32_ALPHA_MASK);
    }

    Ok(header)
}

fn write_pixel_array(image: &Image, dib_header: &DIBHeader) -> Result<Vec<u8>, ImageIOError> {
    match dib_header.compression {
        Compression::Uncompressed => Ok(write_pixel_array_uncompressed(&image)),
        Compression::Bitfields => write_pixel_array_bitfields(&image, &dib_header),
    }
}

fn write_pixel_array_bitfields(image: &Image, dib_header: &DIBHeader) -> Result<Vec<u8>, ImageIOError> {
    let bytes_per_pixel = dib_header.bit_count / 8;
    let mut pixel_array = vec![0 as u8; image.width * image.height * bytes_per_pixel as usize];

    let red_mask_shift = offset_to_far_right(dib_header.red_mask)
        .expect("Expected to get correct shift for red mask");
    let green_mask_shift = offset_to_far_right(dib_header.green_mask)
        .expect("Expected to get correct shift for green mask");
    let blue_mask_shift = offset_to_far_right(dib_header.blue_mask)
        .expect("Expected to get correct shift to blue mask");
    let alpha_mask_shift = if dib_header.alpha_mask != 0 {
        Some(offset_to_far_right(dib_header.alpha_mask)
            .expect("Expected to get correct shift for alpha mask"))
    } else {
        None
    };

    let red_channel_divider = 255 / (dib_header.red_mask >> red_mask_shift) as u8;
    let green_channel_divider = 255 / (dib_header.green_mask >> green_mask_shift) as u8;
    let blue_channel_divider = 255 / (dib_header.blue_mask >> blue_mask_shift) as u8;
    let alpha_channel_divider = alpha_mask_shift.map(|v| 255 / (dib_header.alpha_mask >> v) as u8);

    let mut offset = 0;
    for y in 0..image.height {
        for x in 0..image.width {
            let pixel = &image.get_pixel_bottom_left_origin(x, y);

            let red = (pixel.red / red_channel_divider) as u32;
            let green = (pixel.green / green_channel_divider) as u32;
            let blue = (pixel.blue / blue_channel_divider) as u32;
            let alpha = alpha_channel_divider.map(|d| (pixel.alpha / d) as u32);

            let mut pixel_bits = (red << red_mask_shift) 
                | (green << green_mask_shift) 
                | (blue << blue_mask_shift);

            if let Some(alpha) = alpha {
                let alpha_mask_shift = alpha_mask_shift.expect("Expected alpha channel shift to be present, because divider is set");
                pixel_bits = pixel_bits | (alpha << alpha_mask_shift);
            }

            for n in 0..bytes_per_pixel {
                pixel_array[offset + n as usize] = (pixel_bits.checked_shr(8 * n as u32)
                    .expect("Expected right shift not to underflow, because there are no more than 32 bits per pixel")
                    & 0b1111_1111) as u8;
            }

            offset += bytes_per_pixel as usize;
        }
    }

    Ok(pixel_array)
}

fn write_pixel_array_uncompressed(image: &Image) -> Vec<u8> {
    let row_alignment = (image.width * 3) % 4;
    let width_bytes = image.width * 3 + row_alignment;
    let mut pixel_array = vec![0 as u8; (image.height * width_bytes) as usize];

    let mut offset = 0;
    for y in 0..image.height {
        for x in 0..image.width {
            let pixel = &image.get_pixel_bottom_left_origin(x, y);
            
            pixel_array[offset + 2] = pixel.red;
            pixel_array[offset + 1] = pixel.green;
            pixel_array[offset] = pixel.blue;

            offset += 3;
        }
        
        offset += row_alignment;
    }

    pixel_array
}

fn choose_compression_method(options: &ImageWriterOptions) -> Result<Compression, ImageIOError> {
    let bytes_per_pixel = choose_bits_per_pixel(&options)? / 8;
    Ok(match bytes_per_pixel {
        3 => Compression::Uncompressed,
        2 | 4 => Compression::Bitfields,
        other => return Err(ImageIOError::InvalidOptions {
            description: format!("Unexpected bytes per pixel: {}. Don't know which compression to use", other),
        })
    })
}

fn choose_bits_per_pixel(options: &ImageWriterOptions) -> Result<u8, ImageIOError> {
    options.get_u32(OPTION_BITS_PER_PIXEL, 24).map(|v| v as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_test_image() {
        let test_image = Image::test_image();
        let writer = BMPWriter::new();
        let data = writer.write(&test_image, &ImageWriterOptions::default())
            .expect("failed to write test image");

        assert_eq!(data, vec![
            66, 77, 170, 0, 0, 0, 0, 0, 0, 0, 122, 0, 0, 0, 108, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 1, 0, 
            24, 0, 0, 0, 0, 0, 48, 0, 0, 0, 35, 46, 0, 0, 35, 46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 229, 155, 
            3, 47, 47, 221, 255, 255, 255, 255, 255, 255, 229, 155, 3, 229, 155, 3, 255, 255, 255, 255, 
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255
        ]);
    }

    #[test]
    fn write_with_alignment() {
        let test_image = Image::test_image();
        let mut test_image_3x3 = Image::new(3, 3);
        for y in 0..3 {
            for x in 0..3 {
                test_image_3x3.set_pixel(x, y, test_image.get_pixel(x, y));
            }
        }

        let writer = BMPWriter::new();
        let data = writer.write(&test_image, &ImageWriterOptions::default())
            .expect("failed to write test image");

        assert_eq!(data, vec![
            66, 77, 170, 0, 0, 0, 0, 0, 0, 0, 122, 0, 0, 0, 108, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 1, 0, 24, 
            0, 0, 0, 0, 0, 48, 0, 0, 0, 35, 46, 0, 0, 35, 46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 229, 155, 3, 47, 47, 
            221, 255, 255, 255, 255, 255, 255, 229, 155, 3, 229, 155, 3, 255, 255, 255, 255, 255, 255, 255, 
            255, 255, 255, 255, 255, 255, 255, 255
        ]);
    }

    #[test]
    fn write_16_bit() {
        let test_image = Image::test_image();
        let writer = BMPWriter::new();
        let options = ImageWriterOptions::default()
            .with_option_u32(OPTION_BITS_PER_PIXEL, 16);
        let data = writer.write(&test_image, &options)
            .expect("failed to write test image");

        assert_eq!(data, vec![
            66, 77, 154, 0, 0, 0, 0, 0, 0, 0, 122, 0, 0, 0, 108, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 1, 
            0, 16, 0, 3, 0, 0, 0, 32, 0, 0, 0, 35, 46, 0, 0, 35, 46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            248, 0, 0, 224, 7, 0, 0, 31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 220, 4, 101, 
            217, 255, 255, 255, 255, 220, 4, 220, 4, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255
        ]);
    }

    #[test]
    fn write_32_bit() {
        let test_image = Image::test_image();
        let writer = BMPWriter::new();
        let options = ImageWriterOptions::default()
            .with_option_u32(OPTION_BITS_PER_PIXEL, 32);
        let data = writer.write(&test_image, &options)
            .expect("failed to write test image");

        assert_eq!(data, vec![
            66, 77, 186, 0, 0, 0, 0, 0, 0, 0, 122, 0, 0, 0, 108, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 1, 0, 32, 
            0, 3, 0, 0, 0, 64, 0, 0, 0, 35, 46, 0, 0, 35, 46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 
            255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 
            255, 229, 155, 3, 255, 47, 47, 221, 255, 255, 255, 255, 255, 255, 255, 255, 255, 229, 155, 3, 255, 
            229, 155, 3, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 
            255, 255, 255, 255
        ]);
    }

    #[test]
    fn write_32_alpha() {
        let test_image = Image::test_image_with_alpha();
        let writer = BMPWriter::new();
        let options = ImageWriterOptions::default()
            .with_option_u32(OPTION_BITS_PER_PIXEL, 32)
            .with_option_bool(OPTION_USE_ALPHA_CHANNEL, true);

        let data = writer.write(&test_image, &options)
            .expect("failed to write test image");
        
        assert_eq!(data, vec![
            66, 77, 186, 0, 0, 0, 0, 0, 0, 0, 122, 0, 0, 0, 108, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 1, 0, 32, 
            0, 3, 0, 0, 0, 64, 0, 0, 0, 35, 46, 0, 0, 35, 46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 
            0, 255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 
            229, 155, 3, 100, 229, 155, 3, 50, 255, 255, 255, 0, 255, 255, 255, 0, 229, 155, 3, 255, 229, 
            155, 3, 150, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 255, 255, 
            255, 0
        ]);
    }

    #[test]
    fn write_32_alpha_compose() {
        let test_image = Image::test_image_with_alpha();
        let writer = BMPWriter::new();
        let options = ImageWriterOptions::default()
            .with_option_u32(OPTION_BITS_PER_PIXEL, 32)
            .with_option_bool(OPTION_USE_ALPHA_CHANNEL, false);

        let data = writer.write(&test_image, &options)
            .expect("failed to write test image");
        
        assert_eq!(data, vec![
            66, 77, 186, 0, 0, 0, 0, 0, 0, 0, 122, 0, 0, 0, 108, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 1, 0, 32, 
            0, 3, 0, 0, 0, 64, 0, 0, 0, 35, 46, 0, 0, 35, 46, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 
            255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
            0, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 89, 60, 1, 255, 44, 30, 
            0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 229, 155, 3, 255, 134, 91, 1, 255, 0, 0, 0, 255, 0, 0, 0, 255, 
            0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255
        ]);
    }
}