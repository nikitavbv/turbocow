use core::models::{ImageReader, Image, Pixel, ImageIOError};
use std::str::from_utf8;

#[derive(Debug)]
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
    fn read_raster(&self, header: Header, mut data: &[u8]) -> Vec<Pixel> {
        let mut pixels = Vec::new();
        let normalize = get_normalize_fn(header.max_color_value);
        for _ in 0..header.heigth {
            for _ in 0..header.width {
                data = skip_whitespaces(data);
                let (red, mut new_data) = read_number(data);
                new_data = skip_whitespaces(new_data);
                let (green, mut new_data) = read_number(new_data);
                new_data = skip_whitespaces(new_data);
                let (blue, new_data) = read_number(new_data);
                data = skip_whitespaces(new_data);
                pixels.push(Pixel::from_rgb(normalize(red), normalize(green), normalize(blue)));
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
    box move |x| (255 * x / max_value) as u8
}

fn is_whitespace(char: u8) -> bool {
    // 9 - TAB; 10 - LF; 13 - CR; 32 - SPACE;
    char == 9 || char == 10 || char == 13 ||  char == 32
}

fn read_number(data: &[u8]) -> (usize, &[u8]) {
    let mut i = 0;
    while data.len() > i && !is_whitespace(data[i]) {
        i += 1;
    }
    (from_utf8(&data[0..i]).unwrap().parse::<usize>().unwrap(), &data[i..])
}

fn skip_whitespaces(data: &[u8]) -> &[u8] {
    if data.len() == 0 {
        return data;
    }
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
        skip_comments(&data[(i+1)..])
    } else {
        data
    }
}

fn read_header(mut data: &[u8]) -> (Header, &[u8]) {
    let magic_number = from_utf8(&data[0..2]).expect("Bad data for magic number in PPM header");
    data = &data[2..];
    data = skip_whitespaces(data);
    data = skip_comments(data);
    let (width, mut data) = read_number(data);
    data = skip_whitespaces(data);
    data = skip_comments(data);
    let (heigth, mut data) = read_number(data);
    data = skip_whitespaces(data);
    data = skip_comments(data);
    let (max_color_value, mut data) = read_number(data);
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

impl PPMReader {
    const fn new() -> Self {
        PPMReader {}
    }
}

impl ImageReader for PPMReader {

    fn read(&self, data: &Vec<u8>) -> Result<Image, ImageIOError> {
        let (header, data) = read_header(data);
        let raster_reader = get_raster_reader(header.magic_number.as_str());
        Result::Ok(
            Image {
                width: header.width,
                height: header.heigth,
                pixels: raster_reader.read_raster(header, data),
            }
        )
    }

}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use super::*;

    #[test]
    fn simple_test() {
        let simple_ppm = read("assets/simple.ppm")
            .expect("Failed to load assets/simple.ppm");
        let reader = PPMReader::new();
        let image = reader.read(&simple_ppm).expect("Failed to read the image");

        assert_eq!(image.width, 4);
        assert_eq!(image.height, 4);
        assert_eq!(image.pixels.len(), 16);
        println!("{:?}", image.pixels);
    }

    #[test]
    fn test2() {
        let simple_ppm = read("assets/example1.ppm")
            .expect("Failed to load assets/example1.ppm");
        let reader = PPMReader::new();
        let image = reader.read(&simple_ppm).expect("Failed to read the image");

        println!("{:?}", image.pixels);
        assert_eq!(image.width, 4);
        assert_eq!(image.height, 4);
        assert_eq!(image.pixels.len(), 16);
        assert_eq!(image.get_pixel(1, 1), Pixel::from_rgb(0, 199, 3));
    }
}