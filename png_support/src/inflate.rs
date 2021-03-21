use crate::reader::PNGReaderError;
use bit_vec::BitVec;
use byteorder::{ByteOrder, BigEndian};
use std::iter::FromIterator;
use std::collections::{HashMap, BTreeSet};

struct BitStream {
    position: usize,
    bits: BitVec,
}

impl BitStream {
    fn new(data: &[u8]) -> Self {
        let position = if data.len() > 0 { 7 } else { 0 };
        BitStream {
            position,
            bits: BitVec::from_bytes(data),
        }
    }

    pub fn get_next_bit(&mut self) -> bool {
        // println!("pos: {}", self.position);
        let mut next_position = 0;
        if self.position % 8 == 0 {
            next_position = self.position + 15;
        } else {
            next_position = self.position - 1;
        }
        let bit = self.bits[self.position];
        self.position = next_position;
        bit
    }

    pub fn get_bits(&mut self, amount: usize) -> BitVec {
        let mut result = Vec::new();
        for _ in 0..amount {
            result.insert(0, self.get_next_bit());
            // result.push(self.get_next_bit());
        }
        BitVec::from_iter(&mut result.into_iter())
    }
}

fn print_code(vec: &Vec<bool>) {
    for elem in vec.iter() {
        print!("{}", if *elem { 1 } else { 0 });
    }
}

fn number_to_bits(number: u32, length: usize) -> Vec<bool> {
    let mut buff = [0, 0, 0, 0];
    // NativeEndian::write_u32(&mut buff, number);
    BigEndian::write_u32(&mut buff, number);
    let vec = BitVec::from_bytes(&buff);
    // println!("{:?}", &vec);
    let mut result = Vec::new();
    for i in (32 - length)..32 {
        result.push(vec.get(i).unwrap());
    }
    result
}

fn read_uncompressed_block(data: &[u8]) -> (&[u8], &[u8]) {
    let block_len = BigEndian::read_u16(&data[0..2]) as usize;
    println!("block len: {}", block_len);
    (&data[2..(2 + block_len)], &data[(2 + block_len)..])
}

fn get_len_extra_bits_amount(len: usize) -> usize {
    if len < 8 || len == 28 {
        0
    } else {
        (len - 8) / 4 + 1
    }
}

fn get_dis_extra_bits_amount(dis: usize) -> usize {
    if dis < 4 {
        0
    } else {
        dis / 2 - 1
    }
}

const LEN_LOWER: [usize; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];

fn get_lower_len(len: usize) -> usize {
    LEN_LOWER[len]
}

const DIS_LOWER: [usize; 30] = [1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4007, 6145, 8193, 12289, 16385, 24577];

fn get_lower_dis(dis: usize) -> usize {
    DIS_LOWER[dis]
}

fn bits_to_number(bits: BitVec) -> usize {
    if bits.len() == 0 {
        return 0;
    }
    let r = 8 - bits.len() % 8;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&bits.to_bytes());
    while bytes.len() < 4 {
        bytes.insert(0, 0);
    }
    (BigEndian::read_u32(bytes.as_mut()) >> r) as usize
}

fn get_slice(dis: usize, len: usize, data: &Vec<u8>) -> Vec<u8> {
    let pos = data.len();
    let mut slice = Vec::new();
    let start = pos - dis;
    let mut x = start;
    for _ in 0..len {
        slice.push(data[x]);
        x += 1;
        if x == pos {
            x = start;
        }
    }
    slice
}

fn read_compressed_block(let_len_tree: HashMap<Vec<bool>, usize>, dis_tree: HashMap<Vec<bool>, usize>, data: &mut BitStream) -> Vec<u8> {
    let mut decompressed_data = Vec::new();
    // let mut pos = 0;
    loop {
        let value = read_huffman_code(&let_len_tree, data);
        if value < 256 {
            // println!("value: {}", value);
            decompressed_data.push(value as u8);
            // pos += 1;
        } else if value == 256 {
            println!("end of the block");
            break;
        } else {
            // print!("<{}, ", value);
            let len = value - 257;
            let len = get_lower_len(len) + bits_to_number(data.get_bits(get_len_extra_bits_amount(len)));
            // println!("{:?}", data.get_bits(10));
            // return decompressed_data;
            let dis = read_huffman_code(&dis_tree, data);
            // println!("{}> ", dis);
            let dis = get_lower_dis(dis) + bits_to_number(data.get_bits(get_dis_extra_bits_amount(dis)));
            // println!("pos: {}, len: {}, dis: {}", pos, len, dis);
            decompressed_data.append(&mut get_slice(dis, len, &decompressed_data));
            // pos += len;
        }
    }
    decompressed_data
}

fn generate_huffman_tree(code_lengths: Vec<usize>) -> HashMap<Vec<bool>, usize> {
    let n = code_lengths.len();
    let mut distinct_lengths = BTreeSet::new();
    for len in code_lengths.iter() {
        if *len > 0 {
            distinct_lengths.insert(*len);
        }
    } 
    let mut next_code = 0;
    let mut last_shift = 0;
    let mut codes = HashMap::new();
    for len in distinct_lengths.iter() {
        next_code <<= *len - last_shift;
        last_shift = *len;
        for i in 0..n {
            if code_lengths[i] == *len {
                codes.insert(number_to_bits(next_code, *len), i);
                next_code += 1;
            }
        }
    }
    codes
}

fn read_huffman_code(tree: &HashMap<Vec<bool>, usize>, stream: &mut BitStream) -> usize {
    let mut code = vec![stream.get_next_bit(); 1];
    while !tree.contains_key(&code) {
        // print_code(&code);
        // println!("");
        // code.insert(0, stream.get_next_bit());
        code.push(stream.get_next_bit());
    }
    // print_code(&code);
    // println!("");
    *tree.get(&code).unwrap()
}

fn read_huffman_trees(data: &mut BitStream) -> Result<(HashMap<Vec<bool>, usize>, HashMap<Vec<bool>, usize>), PNGReaderError> {
    let hlit = data.get_bits(5);
    println!("hlit: {:?}", hlit);
    let hlit = (hlit.to_bytes()[0] >> 3) as usize + 257;
    println!("hlit: {}", hlit);

    let hdist = data.get_bits(5);
    println!("hdist: {:?}", &hdist);
    let hdist = (hdist.to_bytes()[0] >> 3) as usize + 1;
    println!("hdist: {}", &hdist);

    let hclen = data.get_bits(4);
    println!("hclen: {:?}", hclen);
    let hclen = (hclen.to_bytes()[0] >> 4) as usize + 4;
    println!("hclen: {}", hclen);

    let mut code_lengths = vec![0; 19];
    let alphabet_order = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
    for i in 0..hclen {
        let length = data.get_bits(3);
        println!("{:?}", &length);
        let length = (length.to_bytes()[0] >> 5) as usize;
        println!("{:?}", &length);
        code_lengths[alphabet_order[i as usize]] = length;
    }
    println!("code_length: {:?}", code_lengths);
    let tree = generate_huffman_tree(code_lengths);
    println!("{:?}", tree);
    println!("position: {}", data.position);
    let mut index = 0;
    let mut codes = Vec::new();
    println!("hlit + hdist: {}", hlit + hdist);
    while index < hlit + hdist {
        let code = read_huffman_code(&tree, data);
        // println!("code: {}", code);
        if code < 15 {
            codes.push(code);
            index += 1;
        } else if code == 16 {
            let times = data.get_bits(2);
            // println!("{:?} times binary", times);
            let times = times.to_bytes()[0] >> 6;
            // println!("{:?} times decimal", times + 3);
            index += times as usize + 3;
            for _ in 0..(times + 3) {
                codes.push(codes.last().unwrap().clone());
            }
        } else if code == 17 {
            let times = data.get_bits(3);
            // println!("{:?} times binary", times);
            let times = times.to_bytes()[0] >> 5;
            // println!("{:?} times decimal", times + 3);
            index += times as usize + 3;
            for _ in 0..(times + 3) {
                codes.push(0);
            }
        } else if code == 18 {
            let times = data.get_bits(7);
            // println!("{:?} times binary", times);
            let times = times.to_bytes()[0] >> 1;
            // println!("{:?} times decimal", times + 11);
            index += times as usize + 11;
            for _ in 0..(times + 11) {
                codes.push(0);
            }
        } else {
            panic!("Error: code must be < 18 but got {}", code);
        }
    }
    println!("{:?}", &codes[0..hlit]);
    let lit_len_tree = generate_huffman_tree(codes[0..hlit].to_vec());
    println!("{:?}", &codes[hlit..]);
    let dis_tree = generate_huffman_tree(codes[hlit..].to_vec());
    Result::Ok((lit_len_tree, dis_tree))
}

fn decompress(mut data: &[u8]) -> Result<Vec<u8>, PNGReaderError> {
    let mut decompressed_data = Vec::new();
    let mut bits = BitStream::new(data);
    loop {
        let bfinal = bits.get_next_bit();
        let btype = if bits.get_next_bit() { 1 } else { 0 };
        let btype = btype + if bits.get_next_bit() { 2 } else { 0 };
        match btype {
            0 => {
                let (block_data, rest_data) = read_uncompressed_block(&data[3..]);
                decompressed_data.extend_from_slice(block_data);
                data = rest_data;
            },
            1 => {
                unimplemented!();
            },
            2 => {
                let (let_len_tree, dis_tree) = read_huffman_trees(&mut bits)?;
                decompressed_data.append(&mut read_compressed_block(let_len_tree, dis_tree, &mut bits));
            },
            3 => panic!("error. reserved"),
            _ => {},
        };
        if bfinal {
            break;
        }
    }
    println!("{:?}", decompressed_data);
    Result::Ok(decompressed_data)
}

pub fn inflate_decompress(data: &[u8]) -> Result<Vec<u8>, PNGReaderError> {
    let mut compression_method_flag = BitVec::from_bytes(&[data[0]]);
    println!("cm_flag: {:?}", compression_method_flag);
    compression_method_flag.and(&BitVec::from_bytes(&[0b00001111]));
    let cm_flag = compression_method_flag.to_bytes()[0];
    println!("cm_flag: {}", cm_flag);

    let mut compression_method_flag = BitVec::from_bytes(&[data[0]]);
    compression_method_flag.and(&BitVec::from_bytes(&[0b11110000]));
    let cinfo_flag = compression_method_flag.to_bytes()[0] >> 4;
    println!("cinfo_flag: {}", cinfo_flag);
    if cinfo_flag > 7 {
        panic!("cinfo_flag > 7: not allowed");
    }

    let mut additional_bits = BitVec::from_bytes(&[data[1]]);
    // TODO: check FCHECK flag (0 - 4 bits)
    let fdict_flag = additional_bits.get(2);
    if let Some(bit) = fdict_flag {
        println!("fdict_flag: {}", bit);
        if bit {
            panic!("Not supported");
            // TODO: read preset dictionary
        }
    }
    additional_bits.and(&BitVec::from_bytes(&[0b11000000]));
    let flevel_flag = additional_bits.to_bytes()[0] >> 6;
    println!("flevel_flag: {}", flevel_flag);
    match flevel_flag {
        2 => println!("all good. continue"),
        _ => panic!("not supported"),
    };
    let decompressed_data = decompress(&data[2..(data.len() - 4)]).unwrap();
    // TODO: read ADLER32 block
    Result::Ok(decompressed_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_to_bits() {
        let bit_vec = number_to_bits(10, 4);
        println!("{:?}", bit_vec);
    }

    #[test]
    fn test2() {
        let mut data = BitStream::new(&[2]);
        println!("{:?}", data.bits);
        let bits = data.get_bits(2);
        println!("{:?}", bits);
        let bits = bits.to_bytes()[0] >> 6;
        println!("{:?}", bits);
    }

    #[test]
    fn test_bits_to_number() {
        let mut bits = BitVec::new();
        bits.push(true);
        bits.push(false);
        bits.push(true);
        bits.push(true);
        bits.push(false);
        assert_eq!(22, bits_to_number(bits));
        let mut bits = BitVec::new();
        bits.push(true);
        bits.push(true);
        bits.push(true);
        bits.push(false);
        bits.push(false);
        bits.push(true);
        bits.push(false);
        bits.push(false);
        bits.push(true);
        assert_eq!(457, bits_to_number(bits));
    }

    #[test]
    fn test_get_len_extra_bits_amount() {
        let diff = 257;
        assert_eq!(get_len_extra_bits_amount(257 - diff), 0);
        assert_eq!(get_len_extra_bits_amount(264 - diff), 0);
        assert_eq!(get_len_extra_bits_amount(265 - diff), 1);
        assert_eq!(get_len_extra_bits_amount(266 - diff), 1);
        assert_eq!(get_len_extra_bits_amount(267 - diff), 1);
        assert_eq!(get_len_extra_bits_amount(268 - diff), 1);
        assert_eq!(get_len_extra_bits_amount(269 - diff), 2);
        assert_eq!(get_len_extra_bits_amount(272 - diff), 2);
        assert_eq!(get_len_extra_bits_amount(273 - diff), 3);
        assert_eq!(get_len_extra_bits_amount(276 - diff), 3);
        assert_eq!(get_len_extra_bits_amount(277 - diff), 4);
        assert_eq!(get_len_extra_bits_amount(280 - diff), 4);
        assert_eq!(get_len_extra_bits_amount(281 - diff), 5);
        assert_eq!(get_len_extra_bits_amount(284 - diff), 5);
        assert_eq!(get_len_extra_bits_amount(285 - diff), 0);
    }

    #[test]
    fn test_get_dis_extra_bits_amount() {
        assert_eq!(get_dis_extra_bits_amount(0), 0);
        assert_eq!(get_dis_extra_bits_amount(3), 0);
        assert_eq!(get_dis_extra_bits_amount(4), 1);
        assert_eq!(get_dis_extra_bits_amount(5), 1);
        assert_eq!(get_dis_extra_bits_amount(6), 2);
        assert_eq!(get_dis_extra_bits_amount(7), 2);
        assert_eq!(get_dis_extra_bits_amount(24), 11);
        assert_eq!(get_dis_extra_bits_amount(27), 12);
        assert_eq!(get_dis_extra_bits_amount(29), 13);
    }

    #[test]
    fn test_get_slice() {
        let vec = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let slice = get_slice(4, 13, &vec);
        assert_eq!(slice, vec![4, 5, 6, 7, 4, 5, 6, 7, 4, 5, 6, 7, 4]);

        let vec = vec![1, 0];
        let slice = get_slice(4, 13, &vec);
        assert_eq!(slice, vec![4, 5, 6, 7, 4, 5, 6, 7, 4, 5, 6, 7, 4]);
    }
}
