use custom_error::custom_error;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pixel {
    red: u8,
    green: u8,
    blue: u8,
}

impl Pixel {

    pub fn zero() -> Self {
        Pixel {
            red: 0,
            green: 0,
            blue: 0,
        }
    }

    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Pixel {
            red,
            green,
            blue,
        }
    }
}

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Pixel>, // starting at top left pixel of the image, pos = y * width + x
}

impl Image {

    pub fn new(width: usize, height: usize) -> Self {
        Image {
            width,
            height,
            pixels: vec![Pixel::zero(); width as usize * height as usize],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        self.pixels[y * self.width + x] = pixel;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel {
        self.pixels[y * self.width + x]
    }

    pub fn set_pixel_bottom_left_origin(&mut self, x: usize, y: usize, pixel: Pixel) {
        self.set_pixel(x, self.height - y - 1, pixel)
    }
}

custom_error! {pub ImageIOError
    FailedToRead {description: String} = "Failed to read image: {description}",
}

pub trait ImageReader {

    fn read(&self, data: &Vec<u8>) -> Result<Image, ImageIOError>;
}

pub trait ImageWriter {

    fn write(&self, image: &Image) -> Result<Vec<u8>, ImageIOError>;
}