use std::collections::HashMap;

#[derive(Clone)]
pub struct Channel {
    pub id: u8,
    pub horizontal_sampling: u8,
    pub vertical_sampling: u8,
    pub quantization_table_id: u8,
}

#[derive(Clone)]
pub struct HuffmanTable {

    pub id: u8,
    pub table_type: HuffmanTableType,
    pub table: HashMap<(u16, u16), u8>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum HuffmanTableType {
    DC,
    AC,
}
