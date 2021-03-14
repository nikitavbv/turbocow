use super::pixel::Pixel;

#[derive(Clone)]
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

    pub fn test_image_with_alpha() -> Self {
        let mut image = Self::new(4, 4);

        let transparent = Pixel::from_rgba(255, 255, 255, 0);

        image.fill(transparent);
        image.set_pixel(1, 1, Pixel::from_rgba(3, 155, 229, 255));
        image.set_pixel(2, 1, Pixel::from_rgba(3, 155, 229, 150));
        image.set_pixel(1, 2, Pixel::from_rgba(3, 155, 229, 100));
        image.set_pixel(2, 2, Pixel::from_rgba(3, 155, 229, 50));

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

    pub fn compose_alpha_over_background(&self, background: &Pixel) -> Self {
        let mut image = Self::new(self.width, self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                image.set_pixel(x, y, self.get_pixel(x, y).compose_alpha_over_background(&background))
            }
        }

        image
    }
}