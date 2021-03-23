pub struct DIBHeader {
    pub width: i32,
    pub height: i32,
    pub bit_count: u16,

    pub compression: Compression,

    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub alpha_mask: u32,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Compression {
    Uncompressed,
    Bitfields,
}

impl Compression {

    pub fn to_dib_header_value(&self) -> u32 {
        use Compression::*;

        match self {
            Uncompressed => 0,
            Bitfields => 3, 
        }
    }
}

// 0b1111100000000000 -> 0b11111
pub fn offset_to_far_right(v: u32) -> Option<u8> {
    if v == 0 {
        return None;
    }

    let mut v = v;
    let mut total_shifts = 0;

    while v & 0b1 != 1 {
        v = v >> 1;
        total_shifts += 1;
    }

    Some(total_shifts)
}