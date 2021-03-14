use std::convert::TryInto;

use custom_error::custom_error;
use byteorder::{ByteOrder, LittleEndian};

use core::models::{Image, ImageIOError, ImageReader, Pixel};

custom_error! {pub BMPReaderError
    InvalidHeader {description: String} = "Invalid header: {description}",
    InvalidDIBHeader {description: String} = "Invalid DIB header: {description}",
    UnexpectedConfiguration {description: String} = "Unexpected configuration: {description}",
    NotImplemented {description: String} = "Not implemented: {description}"
}

pub struct BMPReader {
}

struct Header {
    offset: u32,
}

struct DIBHeader {
    width: i32,
    height: i32,
    bit_count: u16,

    compression: Compression,

    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
}

#[derive(Debug)]
enum Compression {
    Uncompressed,
    Bitfields,
}

impl BMPReader {

    pub fn new() -> Self {
        BMPReader {}
    }
}

impl ImageReader for BMPReader {


    fn read(&self, data: &Vec<u8>) -> Result<Vec<Image>, ImageIOError> {
        let header = read_header(&data[0..14].try_into().map_err(|err| ImageIOError::FailedToRead {
            description: format!("expected to get 14 bytes for header: {}", err),
        })?).map_err(|err| ImageIOError::FailedToRead {
            description: format!("failed to read bmp header: {}", err),
        })?;
        let dib_header = read_dib_header(&data[14..]).map_err(|err| ImageIOError::FailedToRead {
            description: format!("failed to read dib header: {}", err)
        })?;

        read_pixel_array(&data[header.offset as usize..], &dib_header)
            .map_err(|err| ImageIOError::FailedToRead {
                description: format!("failed to read as bmp: {}", err),
            })
            .map(|v| vec![v])
    }
}

fn read_header(header: &[u8; 14]) -> Result<Header, BMPReaderError> {
    // 0 - 2 bytes -  header - "BM"
    if header[0] != 0x42 && header[1] != 0x4D {
        return Err(BMPReaderError::InvalidHeader {
            description: "file does not start with 0x42 0x4D".to_string()
        });
    }

    // 2 - 4 bytes - size of BMP file in bytes
    let _size_in_bytes = LittleEndian::read_u32(&header[2..6]);

    // 6 - 2 bytes - reserved
    // 8 - 2 bytes - reserved
    // 10 - 4 bytes - offset of the byte where the bitmap image datga (pixel array) can be found.
    let offset = LittleEndian::read_u32(&header[10..14]);

    Ok(Header {
        offset
    })
}

fn read_dib_header(header: &[u8]) -> Result<DIBHeader, BMPReaderError> {
    // types mapping:
    // dword - u32
    // long - i32
    // word - u16

    // 0 - 4 bytes - size of this header
    let size_of_header = LittleEndian::read_u32(&header[0..4]);

    if size_of_header != 108 && size_of_header != 124 {
        return Err(BMPReaderError::InvalidDIBHeader {
            description: format!("Unexpected length of DIB header: {} (only BMPv4 and BMPv5 are supported)", size_of_header),
        });
    }

    let width = LittleEndian::read_i32(&header[4..8]);
    let height = LittleEndian::read_i32(&header[8..12]);

    let _planes = LittleEndian::read_u16(&header[12..14]);
    let bit_count = LittleEndian::read_u16(&header[14..16]);

    if bit_count != 24 && bit_count != 16 {
        return Err(BMPReaderError::NotImplemented {
            description: format!("this image uses {} bits", bit_count),
        });
    }

    let compression = match LittleEndian::read_u32(&header[16..20]) {
        0x0000 => Compression::Uncompressed,
        0x0003 => Compression::Bitfields,
        0x0001 => return Err(BMPReaderError::NotImplemented {
            description: "v4 RLE8".to_string(),
        }),
        0x0002 => return Err(BMPReaderError::NotImplemented {
            description: "v4 RLE4".to_string(),
        }),
        0x0004 => return Err(BMPReaderError::NotImplemented {
            description: "v4 JPEG".to_string(),
        }),
        0x0005 => return Err(BMPReaderError::NotImplemented {
            description: "v4 PNG".to_string(),
        }),
        0x000B => return Err(BMPReaderError::NotImplemented {
            description: "v4 CMYK".to_string(),
        }),
        0x000C => return Err(BMPReaderError::NotImplemented {
            description: "v4 CMYK RLE8".to_string(),
        }),
        0x00D => return Err(BMPReaderError::NotImplemented {
            description: "v4 CMYK RLE4".to_string()
        }),
        compression => return Err(BMPReaderError::InvalidDIBHeader {
            description: format!("unknown compression type: {}", compression),
        })
    };
    let _size_image = LittleEndian::read_u32(&header[20..24]); // looks like size of pixel array
    let _xpels_per_meter = LittleEndian::read_i32(&header[24..28]);
    let _ypel_per_meter = LittleEndian::read_i32(&header[28..32]);

    let _clr_used = LittleEndian::read_u32(&header[32..36]);
    let _crl_important = LittleEndian::read_u32(&header[36..40]);
    
    let red_mask = LittleEndian::read_u32(&header[40..44]);
    let green_mask = LittleEndian::read_u32(&header[44..48]);
    let blue_mask = LittleEndian::read_u32(&header[48..52]);

    Ok(DIBHeader {
        width,
        height,
        bit_count,

        compression,

        red_mask,
        green_mask,
        blue_mask,
    })
}

fn read_pixel_array(data: &[u8], dib_header: &DIBHeader) -> Result<Image, BMPReaderError> {
    match dib_header.compression {
        Compression::Uncompressed => read_pixel_array_uncompressed(&data, &dib_header),
        Compression::Bitfields => read_pixel_array_bitfields(&data, &dib_header),
    }
}

fn read_pixel_array_bitfields(data: &[u8], dib_header: &DIBHeader) -> Result<Image, BMPReaderError> {
    let mut image = Image::new(dib_header.width as usize, dib_header.height as usize);
    let bytes_per_pixel = dib_header.bit_count / 8;

    let red_mask_shift = offset_to_far_right(dib_header.red_mask).ok_or(BMPReaderError::InvalidDIBHeader {
        description: format!("Could not determine shift for red mask: {}", dib_header.red_mask),
    })?;
    let green_mask_shift = offset_to_far_right(dib_header.green_mask).ok_or(BMPReaderError::InvalidDIBHeader {
        description: format!("Could not determine shift for green mask: {}", dib_header.green_mask),
    })?;
    let blue_mask_shift = offset_to_far_right(dib_header.blue_mask).ok_or(BMPReaderError::InvalidDIBHeader {
        description: format!("Could not determine shift for blue mask: {}", dib_header.blue_mask),
    })?;

    let red_channel_multiplier = 255 / (dib_header.red_mask >> red_mask_shift) as u8;
    let green_channel_multiplier = 255 / (dib_header.green_mask >> green_mask_shift) as u8;
    let blue_mask_multiplier = 255 / (dib_header.blue_mask >> blue_mask_shift) as u8;

    for y in 0..dib_header.height {
        for x in 0..dib_header.width {
            let offset = ((y * dib_header.width + x) * bytes_per_pixel as i32) as usize;

            let mut pixel: u32 = 0;
            if bytes_per_pixel > 4 {
                return Err(BMPReaderError::UnexpectedConfiguration {
                    description: format!("Too many bytes per pixel: {}", bytes_per_pixel),
                });
            }

            for n in 0..bytes_per_pixel {
                pixel = pixel | ((data[offset + n as usize] as u32).checked_shl(8 * n as u32).unwrap());
            }

            let pixel = Pixel::from_rgb(
                ((pixel & dib_header.red_mask) >> red_mask_shift) as u8 * red_channel_multiplier,
                ((pixel & dib_header.green_mask) >> green_mask_shift) as u8 * green_channel_multiplier,
                ((pixel & dib_header.blue_mask) >> blue_mask_shift) as u8 * blue_mask_multiplier,
            );

            image.set_pixel_bottom_left_origin(x as usize, y as usize, pixel);
        }
    }

    Ok(image)
}

fn read_pixel_array_uncompressed(data: &[u8], dib_header: &DIBHeader) -> Result<Image, BMPReaderError> {
    let mut image = Image::new(dib_header.width as usize, dib_header.height as usize);
    let bytes_per_pixel = dib_header.bit_count / 8;
    if dib_header.bit_count != 24 {
        return Err(BMPReaderError::UnexpectedConfiguration {
            description: "Expected no compression to be used with 24-bit images only".to_string(),
        });
    }

    for y in 0..dib_header.height {
        for x in 0..dib_header.width {
            let offset = ((y * dib_header.width + x) * bytes_per_pixel as i32) as usize;

            image.set_pixel_bottom_left_origin(x as usize, y as usize, Pixel::from_rgb(
                data[offset + 2], 
                data[offset + 1], 
                data[offset]
            ));
        }
    }

    Ok(image)
}

// 0b1111100000000000 -> 0b11111
fn offset_to_far_right(v: u32) -> Option<u8> {
    if v == 0 {
        return None;
    }

    let mut v = v;
    let mut total_shifts = 0;

    while v & 0b1 != 1 {
        v = v >> 1;
        total_shifts += 1;
    }

    Some(total_shifts)
}

#[cfg(test)]
mod tests {
    use std::fs::read;

    use super::*;

    #[test]
    fn test_read_frankfurt() {
        let frankfurt = read("assets/frankfurt.bmp")
            .expect("failed to read test asset");
        
        let reader = BMPReader::new();
        let images = reader.read(&frankfurt).expect("failed to read test image");

        assert_eq!(images.len(), 1);
        let image = &images[0];

        assert_eq!(image.width, 1920);
        assert_eq!(image.height, 1289);

        assert_eq!(image.get_pixel(0, 0), Pixel::from_rgb(48, 36, 86));
        assert_eq!(image.get_pixel(1919, 0), Pixel::from_rgb(50, 28, 75));
        assert_eq!(image.get_pixel(1919, 1288), Pixel::from_rgb(40, 25, 56));
        assert_eq!(image.get_pixel(0, 1288), Pixel::from_rgb(16, 40, 110));
    }

    #[test]
    fn test_read_16_bit() {
        let cat = read("assets/cat.bmp")
            .expect("failed to read test asset");

        let reader = BMPReader::new();
        let images = reader.read(&cat).expect("failed to read test image");

        assert_eq!(images.len(), 1);
        let image = &images[0];

        assert_eq!(image.width, 1920);
        assert_eq!(image.height, 1285);

        assert_eq!(image.get_pixel(0, 0), Pixel::from_rgb(224, 236, 240));
        assert_eq!(image.get_pixel(1919, 0), Pixel::from_rgb(176, 176, 176));
        assert_eq!(image.get_pixel(1919, 1284), Pixel::from_rgb(104, 96, 88));
        assert_eq!(image.get_pixel(0, 1284), Pixel::from_rgb(128, 60, 0));
    }
}