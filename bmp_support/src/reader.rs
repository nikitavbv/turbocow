use std::convert::TryInto;

use custom_error::custom_error;
use byteorder::{ByteOrder, LittleEndian};

use core::models::{Image, ImageIOError, ImageReader, Pixel};

custom_error! {pub BMPReaderError
    InvalidHeader {description: String} = "Invalid header: {description}",
    InvalidDIBHeader {description: String} = "Invalid DIB header: {description}",
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
}

enum Compression {
    Uncompressed,
}

impl BMPReader {

    pub fn new() -> Self {
        BMPReader {}
    }
}

impl ImageReader for BMPReader {


    fn read(&self, data: &Vec<u8>) -> Result<Image, ImageIOError> {
        let header = read_header(&data[0..14].try_into().map_err(|err| ImageIOError::FailedToRead {
            description: format!("expected to get 14 bytes for header: {}", err),
        })?).map_err(|err| ImageIOError::FailedToRead {
            description: format!("failed to read bmp header: {}", err),
        })?;
        let dib_header = read_dib_header(&data[14..]).map_err(|err| ImageIOError::FailedToRead {
            description: format!("failed to read dib header: {}", err)
        })?;

        read_pixel_array(&data[header.offset as usize..], dib_header.width, dib_header.height)
            .map_err(|err| ImageIOError::FailedToRead {
                description: format!("failed to read as bmp: {}", err),
            })
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
    // 6 - 2 bytes - reserved
    // 8 - 2 bytes - reserved
    // 10 - 4 bytes - offset of the byte where the bitmap image datga (pixel array) can be found.
    let offset = LittleEndian::read_u32(&header[10..14]);

    Ok(Header {
        offset
    })
}

fn read_dib_header(header: &[u8]) -> Result<DIBHeader, BMPReaderError> {
    // 0 - 4 bytes - size of this header
    let size_of_header = LittleEndian::read_u32(&header[0..4]);

    if size_of_header != 108 {
        return Err(BMPReaderError::InvalidDIBHeader {
            description: format!("Unexpected length of DIB header: {}", size_of_header),
        });
    }

    let width = LittleEndian::read_i32(&header[4..8]);
    let height = LittleEndian::read_i32(&header[8..12]);

    let _planes = LittleEndian::read_u16(&header[12..14]);
    let bit_count = LittleEndian::read_u16(&header[14..16]);

    if bit_count != 24 {
        return Err(BMPReaderError::NotImplemented {
            description: format!("this image uses {} bits", bit_count),
        });
    }

    let compression = LittleEndian::read_u32(&header[16..20]);
    let _compression = match compression {
        0x0000 => Compression::Uncompressed,
        0x0001 => return Err(BMPReaderError::NotImplemented {
            description: "v4 RLE8".to_string(),
        }),
        0x0002 => return Err(BMPReaderError::NotImplemented {
            description: "v4 RLE4".to_string(),
        }),
        0x0003 => return Err(BMPReaderError::NotImplemented {
            description: "v4 bitfields".to_string(),
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

    Ok(DIBHeader {
        width,
        height,
    })
}

fn read_pixel_array(data: &[u8], width: i32, height: i32) -> Result<Image, BMPReaderError> {
    let mut image = Image::new(width as usize, height as usize);

    for y in 0..height {
        for x in 0..width {
            let offset = ((y * width + x) * 3) as usize;
            image.set_pixel_bottom_left_origin(x as usize, y as usize, Pixel::from_rgb(
                data[offset + 2], 
                data[offset + 1], 
                data[offset]
            ));
        }
    }

    Ok(image)
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
        let image = reader.read(&frankfurt).expect("failed to read test image");

        assert_eq!(image.width, 1920);
        assert_eq!(image.height, 1289);

        assert_eq!(image.get_pixel(0, 0), Pixel::from_rgb(48, 36, 86));
        assert_eq!(image.get_pixel(1919, 0), Pixel::from_rgb(50, 28, 75));
        assert_eq!(image.get_pixel(1919, 1288), Pixel::from_rgb(40, 25, 56));
        assert_eq!(image.get_pixel(0, 1288), Pixel::from_rgb(16, 40, 110));
    }
}