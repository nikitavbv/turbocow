use core::models::io::{ImageReader, ImageIOError, ImageWriter, ImageWriterOptions};
use core::models::image::Image;
use core::models::pixel::Pixel;
use custom_error::custom_error;
use byteorder::{ByteOrder, LittleEndian};
use std::iter::*;
use crate::inflate::inflate_decompress;
use crate::chunk::*;
use crate::filter::*;

custom_error! {pub PNGReaderError
    InvalidSignature {description: String} = "Invalid signature: {description}",
    InvalidBytes {description: String} = "Invalid bytes: {description}",
    UnsupportedOption {description: String} = "Option is unsupported: {description}",
    InvalidChunk {description: String} = "Invalid PNG chunk: {description}",
    InvalidImageType {description: String} = "Invalid PNG image type: {description}",
    BadImageData {description: String} = "Image data is corrupted: {description}",
}

pub struct PNGReader {
}

impl PNGReader {
    const fn new() -> Self {
        PNGReader {}
    }
}

pub struct PNGImage {
    pub ihdr: Option<IHDRChunk>,
    pub sbit: Option<SBITChunk>,
    pub phys: Option<PHYSChunk>,
    pub text: Option<TEXTChunk>,
    pub iccp: Option<ICCPChunk>,
    pub time: Option<TIMEChunk>,
    pub ztxt: Option<ZTXTChunk>,
    pub itxt: Option<ITXTChunk>,
    pub idat: IDATChunk,
}

impl PNGImage {
    pub const fn new() -> Self {
        PNGImage {
            ihdr: Option::None,
            sbit:  Option::None,
            phys:  Option::None,
            text:  Option::None,
            iccp: Option::None,
            time: Option::None,
            ztxt: Option::None,
            itxt: Option::None,
            idat:  IDATChunk::new(),
        }
    }
}

#[derive(Debug)]
pub enum PNGImageType {
    Greyscale,
    Truecolour,
    IndexedColour,
    GreyscaleAlpha,
    TruecolourAlpha,
}

impl PNGImageType {
    pub fn from_number(number: u8) -> Result<PNGImageType, PNGReaderError> {
        match number {
            0 => Result::Ok(PNGImageType::Greyscale),
            2 => Result::Ok(PNGImageType::Truecolour),
            3 => Result::Ok(PNGImageType::IndexedColour),
            4 => Result::Ok(PNGImageType::GreyscaleAlpha),
            6 => Result::Ok(PNGImageType::TruecolourAlpha),
            _ => Result::Err(PNGReaderError::InvalidImageType {
                description: format!("{}", number)
            }),
        }
    }

    pub fn get_samples_amount(&self) -> usize {
        match &self {
            PNGImageType::Greyscale => 1,
            PNGImageType::Truecolour => 3,
            PNGImageType::IndexedColour => 1,
            PNGImageType::GreyscaleAlpha => 2,
            PNGImageType::TruecolourAlpha => 4,
        }
    }
}

fn validate_signature(data: &Vec<u8>) -> Result<(), PNGReaderError> {
    if data[0..8] != [137, 80, 78, 71, 13, 10, 26, 10] {
        Result::Err(PNGReaderError::InvalidSignature {
            description: format!(
                "PNG signature must be [137, 80, 78, 71, 13, 10, 26, 10] but got {:?}",
                &data[0..8]
            )
        })
    } else {
        Result::Ok(())
    }
}

fn truecolor_samples_to_pixels(data: Vec<u8>) -> Vec<Pixel> {
    let mut pixels = Vec::new();
    let mut iter = data.iter();
    loop {
        let mut pixel = Pixel::zero();
        match iter.next() {
            Some(red) => pixel.red = *red,
            None => return pixels,
        };
        match iter.next() {
            Some(green) => pixel.green = *green,
            None => return pixels,
        };
        match iter.next() {
            Some(blue) => pixel.blue = *blue,
            None => return pixels,
        };
        println!("{:?}", pixel);
        pixels.push(pixel);
    }
}

fn truecolor_alpha_samples_to_pixels(data: Vec<u8>) -> Vec<Pixel> {
    let mut pixels = Vec::new();
    let mut iter = data.iter();
    loop {
        let mut pixel = Pixel::zero();
        match iter.next() {
            Some(red) => pixel.red = *red,
            None => return pixels,
        };
        match iter.next() {
            Some(green) => pixel.green = *green,
            None => return pixels,
        };
        match iter.next() {
            Some(blue) => pixel.blue = *blue,
            None => return pixels,
        };
        match iter.next() {
            Some(alpha) => pixel.alpha = *alpha,
            None => return pixels,
        };
        println!("{:?}", pixel);
        pixels.push(pixel);
    }
}

fn convert_to_pixels(ihdr: &IHDRChunk, data: Vec<u8>) -> Result<Vec<Pixel>, PNGReaderError> {
    println!("convert_to_pixels data len: {}", data.len());
    println!("{:?}", &data);
    let image_type = &ihdr.colour_type;
    match image_type {
        PNGImageType::Truecolour => Result::Ok(truecolor_samples_to_pixels(data)),
        PNGImageType::TruecolourAlpha => Result::Ok(
            truecolor_alpha_samples_to_pixels(data)
        ),
        _ => Result::Err(PNGReaderError::UnsupportedOption {
            description: format!("Color type {:?} is not supported yet", image_type)
        }),
    }
}

impl ImageReader for PNGReader {

    fn read(&self, data: &Vec<u8>) -> Result<Vec<Image>, ImageIOError> {
        validate_signature(data).map_err(|err| ImageIOError::FailedToRead {
            description: format!("File is corrupted or this is not a PNG file: {}", err)
        })?;
        let image = read_chunks(&data[8..]).map_err(|err| ImageIOError::FailedToRead {
            description: format!("Bad chunks: {}", err)
        })?;
        let ihdr = &image.ihdr.unwrap();
        let uncompressed_data = inflate_decompress(&image.idat.data[0..]).unwrap();
        let unfiltered_data = unfilter(ihdr, uncompressed_data).map_err(|err| ImageIOError::FailedToRead {
            description: format!("Failed to unfilter data: {}", err)
        })?;
        let pixels = convert_to_pixels(ihdr, unfiltered_data).map_err(|err| ImageIOError::FailedToRead {
            description: format!("Failed to construxt pixels: {}", err)
        })?;
        println!("{:?}", pixels);
        Result::Ok(vec![Image { width: ihdr.width as usize, height: ihdr.height as usize, pixels }])
    }

}

pub struct BMPWriter {
}

impl BMPWriter {

    pub fn new() -> Self {
        BMPWriter {
        }
    }
}

impl ImageWriter for BMPWriter {
    
    fn write(&self, image: &Image, _options: &ImageWriterOptions) -> Result<Vec<u8>, ImageIOError> {
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
    LittleEndian::write_u32(&mut header[16..20], 0);
    LittleEndian::write_u32(&mut header[20..24], image.width as u32 * image.height as u32 * bytes_per_pixel as u32); // image_size
    LittleEndian::write_i32(&mut header[24..28], 11811);
    LittleEndian::write_i32(&mut header[28..32], 11811);

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
    use std::fs::read;
    use super::*;
    use bit_vec::BitVec;
    use std::io::prelude::*;
    use std::fs::File;

    #[test]
    fn test() {
        let shisui_png = read("assets/simple3.png")
            .expect("Failed to load assets/shisui.png");
        let reader = PNGReader::new();
        let images = reader.read(&shisui_png).unwrap();
        let writer = BMPWriter::new();
        let bytes = writer.write(&images[0], &ImageWriterOptions::default()).unwrap();
        let mut res_bmp = File::create("res3.bmp").unwrap();
        res_bmp.write_all(&bytes[0..]).unwrap();
    }

    #[test]
    fn bit_vec_test() {
        let mut vec = BitVec::new();
        vec.push(false);
        vec.push(true);
        vec.push(true);
        println!("<{:?}>", &vec);
        println!("{:?}", &vec.to_bytes());
        assert_eq!(3, vec.len());
    }
}