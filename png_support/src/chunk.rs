use byteorder::{ByteOrder, BigEndian};
use std::str::from_utf8;
use crate::reader::{PNGReaderError, PNGImage, PNGImageType};

#[derive(Debug)]
pub struct IHDRChunk {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub colour_type: PNGImageType,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

#[derive(Debug)]
pub struct SBITChunk {
    sample_depths: Vec<u8>,
}

#[derive(Debug)]
pub struct TEXTChunk {
    text: String
}

#[derive(Debug)]
pub struct PHYSChunk {
    pixels_per_unit_x: u32,
    pixels_per_unit_y: u32,
    unit_specifier: u8,
}

#[derive(Debug)]
pub struct IDATChunk {
    pub data: Vec<u8>,
}

impl IDATChunk {
    pub const fn new() -> Self {
        IDATChunk {
            data: Vec::new()
        }
    }
    pub fn append_from_slice(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    pub fn append_chunk(&mut self, mut chunk: IDATChunk) {
        self.data.append(&mut chunk.data);
    }
}

fn read_ihdr_chunk(data: &[u8]) -> Result<(IHDRChunk, &[u8]), PNGReaderError> {
    let length = BigEndian::read_u32(&data[0..4]) as usize;
    println!("{}", length);
    let chunk_type = from_utf8(&data[4..8]).unwrap();
    println!("{}", chunk_type);
    let ihdr_chunk = IHDRChunk {
        width: BigEndian::read_u32(&data[8..12]),
        height: BigEndian::read_u32(&data[12..16]),
        bit_depth: data[16],
        colour_type: PNGImageType::from_number(data[17])?,
        compression_method: data[18],
        filter_method: data[19],
        interlace_method: data[20]
    };
    println!("{:?}", ihdr_chunk);
    Result::Ok((ihdr_chunk, &data[(length + 12)..]))
}

fn read_sbit_chunk(data: &[u8]) -> Result<(SBITChunk, &[u8]), PNGReaderError> {
    let length = BigEndian::read_u32(&data[0..4]) as usize;
    println!("{}", length);
    let chunk_type = from_utf8(&data[4..8]).unwrap();
    println!("{}", chunk_type);
    let sample_depths = &data[8..(8 + length)];
    println!("{:?}", sample_depths);
    Result::Ok((SBITChunk { sample_depths: sample_depths.to_vec() }, &data[length + 12..]))
}

fn read_phys_chunk(data: &[u8]) -> Result<(PHYSChunk, &[u8]), PNGReaderError> {
    let length = BigEndian::read_u32(&data[0..4]) as usize;
    println!("{}", length);
    let chunk_type = from_utf8(&data[4..8]).unwrap();
    println!("{}", chunk_type);
    let phys_chunk = PHYSChunk {
        pixels_per_unit_x: BigEndian::read_u32(&data[8..12]),
        pixels_per_unit_y: BigEndian::read_u32(&data[12..16]),
        unit_specifier: data[16],
    };
    println!("{:?}", phys_chunk);
    Result::Ok((phys_chunk, &data[length + 12..]))
}

fn read_text_chunk(data: &[u8]) -> Result<(TEXTChunk, &[u8]), PNGReaderError> {
    let length = BigEndian::read_u32(&data[0..4]) as usize;
    println!("{}", length);
    let chunk_type = from_utf8(&data[4..8]).unwrap();
    println!("{}", chunk_type);
    let text = from_utf8(&data[8..(8 + length)]).unwrap();
    println!("{}", text);
    Result::Ok((TEXTChunk { text: text.to_owned() }, &data[length + 12..]))
}

fn read_idat_chunk(data: &[u8]) -> Result<(IDATChunk, &[u8]), PNGReaderError> {
    let length = BigEndian::read_u32(&data[0..4]) as usize;
    println!("{}", length);
    let chunk_type = from_utf8(&data[4..8]).unwrap();
    println!("{}", chunk_type);
    let chunk_data = data[8..(8 + length)].to_vec();
    Result::Ok((IDATChunk { data: chunk_data }, &data[length + 12..]))
}

fn read_iend_chunk(data: &[u8]) -> Result<&[u8], PNGReaderError> {
    let length = BigEndian::read_u32(&data[0..4]) as usize;
    println!("{}", length);
    let chunk_type = from_utf8(&data[4..8]).unwrap();
    println!("{}", chunk_type);
    let len_rest = &data[length + 12..].len();
    println!("rest: {}", len_rest);
    Result::Ok(&data[length + 12..])
}

pub fn read_chunks(mut data: &[u8]) -> Result<PNGImage, PNGReaderError> {
    let mut image = PNGImage::new();
    loop {
        match data[4..8] {
            [73, 69, 78, 68] => {
                let chunks = read_iend_chunk(data)?;
                if chunks.len() > 0 {
                    return Result::Err(PNGReaderError::InvalidChunk {
                        description: "IEND chunk must be last".to_owned()
                    });
                } else if image.ihdr.is_none() {
                    return Result::Err(PNGReaderError::InvalidChunk {
                        description: "Missing IHDR chunk".to_owned()
                    });
                } else {
                    return Result::Ok(image);
                }
            },
            [73, 68, 65, 84] => {
                let (chunk, rest_data) = read_idat_chunk(data)?;
                data = rest_data;
                image.idat.append_chunk(chunk);
            },
            [116, 69, 88, 116] => {
                let (chunk, rest_data) = read_text_chunk(data)?;
                data = rest_data;
                image.text = Option::from(chunk);
            },
            [112, 72, 89, 115] => {
                let (chunk, rest_data) = read_phys_chunk(data)?;
                data = rest_data;
                image.phys = Option::from(chunk);
            },
            [115, 66, 73, 84] => {
                let (chunk, rest_data) = read_sbit_chunk(data)?;
                data = rest_data;
                image.sbit = Option::from(chunk);
            },
            [73, 72, 68, 82] => {
                let (chunk, rest_data) = read_ihdr_chunk(data)?;
                data = rest_data;
                image.ihdr = Option::from(chunk);
            },
            _ => {
                return Result::Err(PNGReaderError::InvalidChunk {
                    description: format!("chunk {:?} not supported or does not exist", &data[4..8])
                });
            }
        };
    }
}