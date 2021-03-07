use core::models::{ImageReader, Image, Pixel};
use std::str::from_utf8;

struct Header {
    magic_number: String,
    width: usize,
    heigth: usize,
    max_color_value: usize,
}

trait RasterReader {
    fn read_raster(&self, header: Header, data: &[u8]) -> Vec<Pixel>;
}

pub struct P3RasterReader {
}

impl P3RasterReader {
    pub fn new() -> Self {
        P3RasterReader {
        }
    }
}

impl RasterReader for P3RasterReader {
    fn read_raster(&self, header: Header, data: &[u8]) -> Vec<Pixel> {
        let pixels = Vec::new();
        let normalize = get_normalize_fn(header.max_color_value);
        for _ in 0..header.heigth {
            for _ in 0..header.width {
                let (red, data) = read_number(data);
                data = skip_whitespaces(data);
                let (green, data) = read_number(data);
                data = skip_whitespaces(data);
                let (blue, data) = read_number(data);
                data = skip_whitespaces(data);
                pixels.push((normalize(red), normalize(green), normalize(blue)));
            }
        }
        pixels
    }
}

fn get_raster_reader(magic_number: &str) -> Box<dyn RasterReader> {
    match magic_number {
        "P3" => box P3RasterReader::new(),
        // "P6" => P6RasterReader::new(),
        _ => panic!("Current PPM reader does not support {} magic number for PPM format.", magic_number),
    }
}

fn get_normalize_fn(max_value: usize) -> Box<dyn Fn(usize) -> u8> {
    box |x| (255 * x / max_value) as u8
}

fn is_whitespace(char: u8) -> bool {
    // 9 - TAB; 10 - LF; 13 - CR; 32 - SPACE;
    char == 9 || char == 10 || char == 13 ||  char == 32
}

fn read_number(data: &[u8]) -> (usize, &[u8]) {
    let mut i = 0;
    while data.len() > 0 && !is_whitespace(data[i]) {
        i += 1;
    }
    (from_utf8(&data[0..i]).unwrap().parse::<usize>().unwrap(), &data[i..])
}

fn skip_whitespaces(data: &[u8]) -> &[u8] {
    let mut i = 0;
    while is_whitespace(data[i]) {
        i += 1;
    }
    &data[i..]
}

fn skip_comments(data: &[u8]) -> &[u8] {
    // 35 - #
    // 10 - LF
    if data[0] == 35 {
        let mut i = 0;
        while data[i] != 10 {
            i += 1;
        }
        &data[(i+1)..]
    } else {
        data
    }
}

fn read_header(mut data: &[u8]) -> (Header, &[u8]) {
    let magic_number = from_utf8(&data[0..2]).expect("Bad data for magic number in PPM header");
    data = &data[2..];
    data = skip_whitespaces(data);
    data = skip_comments(data);
    let (width, data) = read_number(data);
    data = skip_whitespaces(data);
    data = skip_comments(data);
    let (heigth, data) = read_number(data);
    data = skip_whitespaces(data);
    data = skip_comments(data);
    let (max_color_value, data) = read_number(data);
    data = skip_whitespaces(data);
    data = skip_comments(data);
    (Header {
        magic_number: magic_number.to_owned(),
        width,
        heigth,
        max_color_value,
    }, data)
}

pub struct PPMReader {
}

impl ImageReader for PPMReader {

    fn read(&self, mut data: &[u8]) -> Image {
        let (header, data) = read_header(data);
        let raster_reader = get_raster_reader(header.magic_number.as_str());
        Image {
            width: header.width,
            height: header.heigth,
            pixels: raster_reader.read_raster(header, data),
        }
    }

}