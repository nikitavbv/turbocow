use core::models::{Image, ImageIOError, ImageWriter, Pixel};

use byteorder::{LittleEndian, ByteOrder};
pub struct BMPWriter {
}

impl BMPWriter {

    pub fn new() -> Self {
        BMPWriter {
        }
    }
}

impl ImageWriter for BMPWriter {
    
    fn write(&self, image: &Image) -> Result<Vec<u8>, ImageIOError> {
        let image = image.compose_alpha_over_background(Pixel::black());

        let mut output = vec![];

        let mut dib_header = write_dib_header(&image);
        let dib_header_size = dib_header.len() as u32;

        output.append(&mut dib_header);
        output.append(&mut write_pixel_array(&image));

        let mut header = write_header(&output, dib_header_size);
        header.append(&mut output);

        let output = header;

        Ok(output)
    }
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

fn write_dib_header(image: &Image) -> Vec<u8> {
    let header_len = 108;
    let mut header = vec![0; header_len];

    let bytes_per_pixel = 3;

    LittleEndian::write_u32(&mut header[0..4], header_len as u32);
    LittleEndian::write_i32(&mut header[4..8], image.width as i32);
    LittleEndian::write_i32(&mut header[8..12], image.height as i32);
    LittleEndian::write_u16(&mut header[12..14], 1);
    LittleEndian::write_u16(&mut header[14..16], bytes_per_pixel * 8);
    LittleEndian::write_u32(&mut header[16..20], 0); // no compression
    LittleEndian::write_u32(&mut header[20..24], image.width as u32 * image.height as u32 * bytes_per_pixel as u32); // image_size
    LittleEndian::write_i32(&mut header[24..28], 11811); // xpels_per_meter
    LittleEndian::write_i32(&mut header[28..32], 11811); // ypels_per_meter

    header
}

fn write_pixel_array(image: &Image) -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_test_image() {
        let test_image = Image::test_image();
        let writer = BMPWriter::new();
        let data = writer.write(&test_image).expect("failed to write test image");

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
        let data = writer.write(&test_image).expect("failed to write test image");

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
}