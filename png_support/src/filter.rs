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

fn get_left(pos: usize, samples_amount: usize, data: &Vec<u8>) -> u8 {
    if pos < samples_amount {
        return 0
    }
    data[data.len() - samples_amount]
}

fn get_upper(width: usize, samples_amount: usize, data: &Vec<u8>) -> u8 {
    if data.len() < width * samples_amount {
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

fn unfilter_sub(width: usize, samples_amount: usize, iter: &mut Iter<u8>) -> Vec<u8> {
    let mut res = Vec::new();
    for _ in 0..samples_amount {
        res.push(*iter.next().unwrap());
    }
    for _ in 0..((width - 1) * samples_amount) {
        res.push((res[res.len() - samples_amount] as u16 + *iter.next().unwrap() as u16) as u8);
    }
    res
}

fn unfilter_up(width: usize, samples_amount: usize, iter: &mut Iter<u8>) -> Vec<u8> {
    let mut scanline = Vec::new();
    for _ in 0..(width * samples_amount) {
        scanline.push(*iter.next().unwrap());
    }
    for i in (width as usize - 2)..=0 {
        for j in 0..samples_amount {
            scanline[i + j] += scanline[i + samples_amount + j];
        }
    }
    scanline
}

fn unfilter_average(width: usize, samples_amount: usize, iter: &mut Iter<u8>) -> Vec<u8> {
    let mut res = Vec::new();
    let mut prev = vec![0; samples_amount];
    let mut cur = Vec::new();
    let mut readed = 0;
    for _ in 0..samples_amount {
        cur.push(*iter.next().unwrap());
    }
    readed += samples_amount;
    for _ in 0..(width - 1) {
        let mut next = Vec::new();
        for _ in 0..samples_amount {
            next.push(*iter.next().unwrap());
        }
        readed += samples_amount;
        for i in 0..samples_amount {
            res.push((cur[i] as u16 + ((prev[i] as u16 + next[i] as u16) / 2)) as u8);
        }
        prev = cur;
        cur = next;
    }
    for i in 0..samples_amount {
        res.push((cur[i] as u16 + ((prev[i] as u16) / 2)) as u8);
    }
    println!("readed: {}", readed);
    res
}

fn unfilter_peath(width: usize, samples_amount: usize, iter: &mut Iter<u8>) -> Vec<u8> {
    let mut res = Vec::new();
    for i in 0..(width * samples_amount) {
        let val = *iter.next().unwrap();
        res.push((val as u16 + predict_paeth(
            get_left(i, samples_amount, &res),
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
                    res.append(&mut unfilter_sub(width, samples_amount, &mut iter));
                } else if *filter_type == 2 {
                    res.append(&mut unfilter_up(width, samples_amount, &mut iter));
                } else if *filter_type == 3 {
                    res.append(&mut unfilter_average(width, samples_amount, &mut iter));
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