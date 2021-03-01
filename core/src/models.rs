pub type Pixel = (u8, u8, u8); // rgb

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Vec<Pixel>>, // each Vec<Pixel> is a row, i.e. pixels is a Vec of rows.
}

pub trait ImageReader {

    fn read(&self, data: &[u8]) -> Image;
}

pub trait ImageWriter {

    fn write(&self, image: &Image) -> Vec<u8>;
}