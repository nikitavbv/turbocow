use custom_error::custom_error;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Pixel {

    pub fn zero() -> Self {
        Self::black()
    }

    pub fn white() -> Self {
        Self::from_rgb(255, 255, 255)
    }

    pub fn black() -> Self {
        Self::from_rgb(0, 0, 0)
    }

    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::from_rgba(red, green, blue, 255)
    }

    pub fn from_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Pixel {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn compose_alpha_over_background(&self, background: &Pixel) -> Self {
        let foreground_multiplier = self.alpha as f32 / 255.0;
        let background_multiplier = (255 - self.alpha) as f32 / 255.0;
        Self::from_rgb(
            (self.red as f32 * foreground_multiplier + background.red as f32 * background_multiplier) as u8,
            (self.green as f32 * foreground_multiplier + background.green as f32 * background_multiplier) as u8,
            (self.blue as f32 * foreground_multiplier + background.blue as f32 * background_multiplier) as u8,
        )
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

    pub fn test_image() -> Self {
        let mut image = Self::new(4, 4);

        let white = Pixel::from_rgb(255, 255, 255);
        let blue = Pixel::from_rgb(3, 155, 229);
        let red = Pixel::from_rgb(221, 47, 47);

        image.fill(white);
        image.set_pixel(1, 1, blue);
        image.set_pixel(2, 1, blue);
        image.set_pixel(1, 2, blue);
        image.set_pixel(2, 2, red);

        image
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        self.pixels[y * self.width + x] = pixel;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel {
        self.pixels[y * self.width + x]
    }

    pub fn set_pixel_bottom_left_origin(&mut self, x: usize, y: usize, pixel: Pixel) {
        self.set_pixel(x, self.height - 1 - y, pixel)
    }

    pub fn get_pixel_bottom_left_origin(&self, x: usize, y: usize) -> Pixel {
        self.get_pixel(x, self.height - 1 - y)
    }

    pub fn fill(&mut self, color: Pixel) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x, y, color.clone());
            } 
        }
    }

    pub fn compose_alpha_over_background(&self, background: Pixel) -> Self {
        let mut image = Self::new(self.width, self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                image.set_pixel(x, y, self.get_pixel(x, y).compose_alpha_over_background(&background))
            }
        }

        image
    }
}

custom_error! {pub ImageIOError
    FailedToRead {description: String} = "Failed to read image: {description}",
}

pub trait ImageReader {

    fn read(&self, data: &Vec<u8>) -> Result<Vec<Image>, ImageIOError>;
}

pub trait ImageWriter {

    fn write(&self, image: &Image) -> Result<Vec<u8>, ImageIOError>;
}
