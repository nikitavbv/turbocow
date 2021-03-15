use core::models::pixel::Pixel;

#[derive(Clone)]
pub struct ColorTable {

    pub size: usize, // size of table in bytes
    pub colors: Vec<Pixel>,
}

pub (crate) struct ImageData {

    pub size: usize, // size of this block in bytes
    pub pixels: Vec<Pixel>,
}

pub fn init_dictionary(dictionary: &mut Vec<Vec<Pixel>>, color_table: &ColorTable) -> (usize, usize) {
    dictionary.clear();
    
    for i in 0..color_table.colors.len() {
        dictionary.push(vec![color_table.colors[i].clone()]);
    }

    let clear_index = dictionary.len();
    dictionary.push(Vec::new());

    let end_index = dictionary.len();
    dictionary.push(Vec::new());

    (clear_index, end_index)
}

pub fn should_increase_code_size(dictionary: &Vec<Vec<Pixel>>, code_size: u8) -> bool {
    dictionary.len() == 2u32.pow(code_size as u32) as usize && code_size < 12
}