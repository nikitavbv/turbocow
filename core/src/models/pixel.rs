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

    pub fn with_alpha_channel(&self, alpha: u8) -> Self {
        Self::from_rgba(self.red, self.green, self.blue, alpha)
    }
}