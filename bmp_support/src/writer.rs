use core::models::{Image, ImageIOError, ImageWriter};

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
        let mut output = vec![];

        let mut dib_header = write_dib_header(&image);
        let dib_header_size = dib_header.len() as u32;

        output.append(&mut dib_header);
        output.append(&mut write_pixel_array(&image));

        let mut header = write_header(&image, &output, dib_header_size);
        header.append(&mut output);

        let output = header;

        Ok(output)
    }
}

fn write_header(image: &Image, data: &Vec<u8>, dib_header_size: u32) -> Vec<u8> {
    let image_size = 14 + data.len();
    let mut header = vec![0; 14];

    header[0] = 0x42;
    header[1] = 0x4D;

    LittleEndian::write_u32(&mut header[2..6], image_size as u32);
    LittleEndian::write_u32(&mut header[10..14], dib_header_size + 14);

    println!("dib header size is {}", dib_header_size);

    header
}

fn write_dib_header(image: &Image) -> Vec<u8> {
    let header_len = 108;
    let mut header = vec![0; header_len];

    LittleEndian::write_u32(&mut header[0..4], header_len as u32);
    LittleEndian::write_i32(&mut header[4..8], image.width as i32);
    LittleEndian::write_i32(&mut header[8..12], image.height as i32);
    LittleEndian::write_u16(&mut header[14..16], 24 as u16);

    header
}

fn write_pixel_array(image: &Image) -> Vec<u8> {
    let mut pixel_array = vec![0 as u8; (image.width * image.height * 3) as usize];

    for y in 0..image.height {
        for x in 0..image.width {
            let offset = ((y * image.width + x) * 3) as usize;
            let pixel = &image.get_pixel_bottom_left_origin(x, y);
            
            pixel_array[offset + 2] = pixel.red;
            pixel_array[offset + 1] = pixel.green;
            pixel_array[offset] = pixel.blue;
        }
    }

    pixel_array
}

#[cfg(test)]
mod tests {
    use std::fs::write;

    use super::*;

    #[test]
    fn write_test_image() {
        let test_image = Image::test_image();

        let writer = BMPWriter::new();
        let data = writer.write(&test_image).expect("failed to write test image");

        write("assets/result.bmp", data).expect("failed to save test image to file");
    }
}