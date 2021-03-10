use core::models::{ImageReader, Image, ImageIOError};
use custom_error::custom_error;
use byteorder::{LittleEndian, ByteOrder, BigEndian};
use std::str::from_utf8;

custom_error! {pub PNGReaderError
    InvalidSignature {description: String} = "Invalid signature: {description}",
    InvalidBytes {description: String} = "Invalid bytes: {description}",
}

pub struct PNGReader {
}

impl PNGReader {
    const fn new() -> Self {
        PNGReader {}
    }
}

#[derive(Debug)]
struct IHDRChunk {
    width: u32,
    height: u32,
    bit_depth: u8,
    colour_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

#[derive(Debug)]
struct SBITChunk {
    sample_depths: Vec<u8>,
}

#[derive(Debug)]
struct TEXTChunk {
    text: String
}

#[derive(Debug)]
struct PHYSChunk {
    pixels_per_unit_x: u32,
    pixels_per_unit_y: u32,
    unit_specifier: u8,
}

#[derive(Debug)]
struct IDATChunk {
    data: Vec<u8>,
}

fn bytes_to_u32(data: &[u8]) -> Result<u32, PNGReaderError> {
    if data.len() != 4 {
        Result::Err(PNGReaderError::InvalidBytes {
            description: format!("Unable toparse number from bytes: {:?}", data)
        })
    } else {
        Result::Ok(0)
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

fn read_IHDR_chunk(data: &[u8]) -> Result<(IHDRChunk, &[u8]), PNGReaderError> {
    let length = BigEndian::read_u32(&data[0..4]) as usize;
    println!("{}", length);
    let chunk_type = from_utf8(&data[4..8]).unwrap();
    println!("{}", chunk_type);
    if data[4..8] != [73, 72, 68, 82] {
        panic!("IHDR chunk must be first. but got: {:?}", &data[4..8]);
    }
    let ihdr_chunk = IHDRChunk {
        width: BigEndian::read_u32(&data[8..12]),
        height: BigEndian::read_u32(&data[12..16]),
        bit_depth: data[16],
        colour_type: data[17],
        compression_method: data[18],
        filter_method: data[19],
        interlace_method: data[20]
    };
    println!("{:?}", ihdr_chunk);
    Result::Ok((ihdr_chunk, &data[(length + 12)..]))
}

fn read_sBIT_chunk(data: &[u8]) -> Result<(SBITChunk, &[u8]), PNGReaderError> {
    let length = BigEndian::read_u32(&data[0..4]) as usize;
    println!("{}", length);
    let chunk_type = from_utf8(&data[4..8]).unwrap();
    println!("{}", chunk_type);
    if data[4..8] != [115, 66, 73, 84] {
        panic!("Expect [115, 66, 73, 84] chank type for sBIT, but got: {:?}", &data[4..8]);
    }
    let sample_depths = &data[8..(8 + length)];
    println!("{:?}", sample_depths);
    Result::Ok((SBITChunk { sample_depths: sample_depths.to_vec() }, &data[length + 12..]))
}

fn read_pHYs_chunk(data: &[u8]) -> Result<(PHYSChunk, &[u8]), PNGReaderError> {
    let length = BigEndian::read_u32(&data[0..4]) as usize;
    println!("{}", length);
    let chunk_type = from_utf8(&data[4..8]).unwrap();
    println!("{}", chunk_type);
    if data[4..8] != [112, 72, 89, 115] {
        panic!("Expect [112, 72, 89, 115] chank type for pHYs, but got: {:?}", &data[4..8]);
    }
    let pHYs_chunk = PHYSChunk {
        pixels_per_unit_x: BigEndian::read_u32(&data[8..12]),
        pixels_per_unit_y: BigEndian::read_u32(&data[12..16]),
        unit_specifier: data[16],
    };
    println!("{:?}", pHYs_chunk);
    Result::Ok((pHYs_chunk, &data[length + 12..]))
}

fn read_tEXt_chunk(data: &[u8]) -> Result<(TEXTChunk, &[u8]), PNGReaderError> {
    let length = BigEndian::read_u32(&data[0..4]) as usize;
    println!("{}", length);
    let chunk_type = from_utf8(&data[4..8]).unwrap();
    println!("{}", chunk_type);
    if data[4..8] != [116, 69, 88, 116] {
        panic!("Expect [116, 69, 88, 116] chank type for tEXt, but got: {:?}", &data[4..8]);
    }
    let text = from_utf8(&data[8..(8 + length)]).unwrap();
    println!("{}", text);
    Result::Ok((TEXTChunk { text: text.to_owned() }, &data[length + 12..]))
}

fn read_IDAT_chunk(data: &[u8]) -> Result<(IDATChunk, &[u8]), PNGReaderError> {
    let length = BigEndian::read_u32(&data[0..4]) as usize;
    println!("{}", length);
    let chunk_type = from_utf8(&data[4..8]).unwrap();
    println!("{}", chunk_type);
    if data[4..8] != [73, 68, 65, 84] {
        panic!("Expect [73, 68, 65, 84] chank type for IDAT, but got: {:?}", &data[4..8]);
    }
    let chunk_data = data[8..(8 + length)].to_vec();
    Result::Ok((IDATChunk { data: chunk_data }, &data[length + 12..]))
}

fn read_IEND_chunk(data: &[u8]) -> Result<&[u8], PNGReaderError> {
    let length = BigEndian::read_u32(&data[0..4]) as usize;
    println!("{}", length);
    let chunk_type = from_utf8(&data[4..8]).unwrap();
    println!("{}", chunk_type);
    if data[4..8] != [73, 69, 78, 68] {
        panic!("Expect [73, 69, 78, 68] chank type for IEND, but got: {:?}", &data[4..8]);
    }
    let len_rest = &data[length + 12..].len();
    println!("rest: {}", len_rest);
    Result::Ok(&data[length + 12..])
}

fn read_chunks(data: &Vec<u8>) -> Result<(), PNGReaderError> {
    Result::Ok(())
}

impl ImageReader for PNGReader {

    fn read(&self, data: &Vec<u8>) -> Result<Vec<Image>, ImageIOError> {
        validate_signature(data).map_err(|err| ImageIOError::FailedToRead {
            description: format!("File is corrupted or this is not a PNG file: {}", err)
        })?;
        Result::Ok(Vec::new())
    }

}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use super::*;

    #[test]
    fn test_IHDR_chunk() {
        let shisui_png = read("assets/shisui.png")
            .expect("Failed to load assets/shisui.png");
        validate_signature(&shisui_png).unwrap();
        let data = &shisui_png[8..];
        let (_, data) = read_IHDR_chunk(data).unwrap();
        let (_, data) = read_sBIT_chunk(data).unwrap();
        let (_, data) = read_pHYs_chunk(data).unwrap();
        let (_, data) = read_tEXt_chunk(data).unwrap();
        let (_, data) = read_IDAT_chunk(data).unwrap();
        let (_, data) = read_IDAT_chunk(data).unwrap();
        let (_, data) = read_IDAT_chunk(data).unwrap();
        let _ = read_IEND_chunk(data).unwrap();
    }
}