use crate::reader::{PNGReaderError};
use crate::chunk::IHDRChunk;
use std::slice::Iter;

fn predict_paeth(a: u8, b: u8, c: u8) -> u8 {
    let a = a as i16;
    let b = b as i16;
    let c = c as i16;
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();
    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

fn get_left(width: usize, samples_amount: usize, data: &Vec<u8>) -> u8 {
    if data.len() % (width * samples_amount) == 0 {
        return 0;
    }
    data[data.len() - samples_amount]
}

fn get_upper(width: usize, samples_amount: usize, data: &Vec<u8>) -> u8 {
    if data.len() < width * samples_amount {
        println!("return 0. data.len: {}, width * sampes_amount: {}", data.len(), width * samples_amount);
        return 0;
    }
    data[data.len() - samples_amount * width]
}

fn get_upper_left(pos: usize, width: usize, samples_amount: usize, data: &Vec<u8>) -> u8 {
    if pos == 0 || data.len() < width * samples_amount {
        return 0;
    }
    data[data.len() - samples_amount * (width + 1)]
}

fn unfilter_none(width: usize, samples_amount: usize, iter: &mut Iter<u8>) -> Vec<u8> {
    let mut res = Vec::new();
    for _ in 0..(width * samples_amount) {
        res.push(*iter.next().unwrap());
    }
    res
}

fn unfilter_sub(width: usize, samples_amount: usize, iter: &mut Iter<u8>, res: &mut Vec<u8>) {
    for _ in 0..samples_amount {
        res.push(*iter.next().unwrap());
    }
    for _ in 0..((width - 1) * samples_amount) {
        res.push((res[res.len() - samples_amount] as u16 + *iter.next().unwrap() as u16) as u8);
    }
}

fn unfilter_up(width: usize, samples_amount: usize, iter: &mut Iter<u8>, res: &mut Vec<u8>) {
    for _ in 0..width {
        for _ in 0..samples_amount {
            res.push((*iter.next().unwrap() as u16 + get_upper(width, samples_amount, &res) as u16) as u8);
        }
    }
}

fn unfilter_average(width: usize, samples_amount: usize, iter: &mut Iter<u8>, res: &mut Vec<u8>) {
    for _ in 0..(width * samples_amount) {
        res.push((*iter.next().unwrap() as u16 + ((get_left(width, samples_amount, res) + get_upper(width, samples_amount, res)) as u16 / 2)) as u8);
    }
}

fn unfilter_peath(width: usize, samples_amount: usize, iter: &mut Iter<u8>) -> Vec<u8> {
    let mut res = Vec::new();
    for i in 0..(width * samples_amount) {
        let val = *iter.next().unwrap();
        res.push((val as u16 + predict_paeth(
            get_left(width, samples_amount, &res),
            get_upper(width as usize, samples_amount, &res),
            get_upper_left(i, width as usize, samples_amount, &res)
        ) as u16) as u8);
    }
    res
}

pub fn unfilter(ihdr: &IHDRChunk, data: Vec<u8>) -> Result<Vec<u8>, PNGReaderError> {
    let filter_method = ihdr.filter_method;
    if filter_method != 0 {
        return Result::Err(PNGReaderError::UnsupportedOption {
            description: format!("Filter method {} is not supported. Only 0 filter method allowed", filter_method)
        });
    }
    let width = ihdr.width as usize;
    let heigth = ihdr.height as usize;
    let samples_amount = ihdr.colour_type.get_samples_amount();
    if data.len() - heigth != width * heigth * samples_amount {
        return Result::Err(PNGReaderError::BadImageData {
            description: format!("Bad samples amount. Expected: {} but got {}", width * heigth * samples_amount, data.len() - heigth)
        });
    }
    println!("unfilter data len: {}. width: {}", data.len(), width);
    let mut k = 0;
    for _ in 0..heigth {
        print!("{} ", data[k]);
        k += 1;
        for _ in 0..(width * 3) {
            print!("{} ", data[k]);
            k += 1;
        }
        println!("");
    }
    let mut iter = data.iter();
    let mut res = Vec::new();
    let mut scanline_count = 1;
    loop {
        println!("scanline_count: {}", scanline_count);
        match iter.next() {
            Some(filter_type) => {
                println!("filter_type: {}", filter_type);
                if *filter_type == 0 {
                    res.append(&mut unfilter_none(width, samples_amount, &mut iter));
                } else if *filter_type == 1 {
                    unfilter_sub(width, samples_amount, &mut iter, &mut res);
                } else if *filter_type == 2 {
                    unfilter_up(width, samples_amount, &mut iter, &mut res);
                } else if *filter_type == 3 {
                    unfilter_average(width, samples_amount, &mut iter, &mut res);
                } else if *filter_type == 4 {
                    res.append(&mut unfilter_peath(width, samples_amount, &mut iter));
                } else {
                    panic!("Unsupported filter type {}", filter_type);
                }
            },
            None => break,
        }
        scanline_count += 1;
    }
    Result::Ok(res)
}