extern crate image;
extern crate byteorder;

use std::path::Path;
use std::process::exit;
use std::io::prelude::*;
use std::fs::File;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use std::io::Cursor;

use image::*;

fn main() {
    let mut buffer = Vec::new();
    let mut f = File::open("to_hide.bin").unwrap();
    f.read_to_end(&mut buffer).unwrap();

    steg("test.jpg", buffer);
    let mut buf = Vec::new();
    unsteg("test.out.png", &mut buf);
}

fn steg(path: &str, bytes: Vec<u8>) {
    let mut img = image::open(&Path::new(path)).unwrap();
    let (dim_x, dim_y) = img.dimensions();

    if dim_x * dim_y * 3 / 8 <= bytes.len() as u32 {
        println!("Image needs more pixels !");
        exit(1);
    }

    let mut img_buf = img.as_mut_rgb8().unwrap();
    let mut it = ImageIterator::new(img_buf);

    // Add header (payload size)
    let mut size_bytes = vec![];
    size_bytes.write_u32::<BigEndian>(dim_x * dim_y).unwrap();
    println!("{:?}", size_bytes);
    hide_bytes(img_buf, &mut it, size_bytes);

    // Add actual payload
    // hide_bytes(img_buf, &mut it, bytes);

    img_buf.save(&Path::new(&format!("{}.png", "test.out"))).unwrap();
}

fn hide_bytes(img_buf: &mut RgbImage, it: &mut ImageIterator, bytes: Vec<u8>) {
    for byte in bytes {
        for bit in 0..8 {
            let bit_to_hide = ((byte >> bit) & 1) as u8;

            let next = it.next().unwrap();
            let mut pixel = img_buf.get_pixel_mut(next.x, next.y);

            let chan_index = next.channel as usize;
            pixel.data[chan_index] = pixel.data[chan_index] & 0xFE | bit_to_hide;
        }
    }
}

fn unsteg(path: &str, bytes: &mut Vec<u8>) {
    let img = image::open(&Path::new(path)).unwrap();
    let img_buf = img.as_rgb8().unwrap();
    let mut it = ImageIterator::new(img_buf);

    // Get payload header (payload size)
    let mut size_bytes = vec![0, 0, 0, 0];
    for byte_index in 0..4 {
        let mut byte_building: u8 = 0;

        for bit_index in 0..8 {
            let next = it.next().unwrap();
            let pixel = img_buf.get_pixel(next.x, next.y);
            let chan_index = next.channel as usize;

            if pixel.data[chan_index] & (1 << 0) == 1 {
                byte_building |= 1 << bit_index;
            }
        }
        size_bytes[byte_index] = byte_building;
    }
    println!("{:?}",size_bytes);
    let mut rdr = Cursor::new(size_bytes);
    let size = rdr.read_u32::<BigEndian>().unwrap();
    println!("{:?}", size);
}

#[derive(Debug)]
struct ImageCoordinate {
    x: u32,
    y: u32,
    channel: u8,
}

struct ImageIterator {
    max_x: u32,
    max_y: u32,
    curr_x: u32,
    curr_y: u32,
    curr_channel: u8,
}

impl ImageIterator {
    pub fn new(img_buf: &RgbImage) -> Self {
        let (dim_x, dim_y) = img_buf.dimensions();
        ImageIterator {
            max_x: dim_x,
            max_y: dim_y,
            curr_x: 0,
            curr_y: 0,
            curr_channel: 0,
        }
    }
}

impl Iterator for ImageIterator {
    type Item = ImageCoordinate;

    fn next(&mut self) -> Option<ImageCoordinate> {
        let coordinate = ImageCoordinate {
            x : self.curr_x,
            y : self.curr_y,
            channel : self.curr_channel
        };

        self.curr_channel += 1;
        if self.curr_channel >= 3 {
            self.curr_channel = 0;
            self.curr_x += 1;

            if self.curr_x >= self.max_x {
                self.curr_x = 0;
                self.curr_y += 1;

                if self.curr_y >= self.max_y {
                    panic!();
                }
            }
        }
        Some(coordinate)
    }
}
