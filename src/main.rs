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
    let mut bytes_to_hide = Vec::new();
    let mut f = File::open("to_hide.bin").unwrap();
    f.read_to_end(&mut bytes_to_hide).unwrap();

    steg("test.jpg", "test.out.png", bytes_to_hide);
    let mut bytes_read = Vec::new();
    unsteg("test.out.png", &mut bytes_read);
}

fn steg(path_input: &str, path_output: &str, bytes_to_hide: Vec<u8>) {
    let mut img = image::open(&Path::new(path_input)).unwrap();
    let (dim_x, dim_y) = img.dimensions();

    if dim_x * dim_y * 3 / 8 <= bytes_to_hide.len() as u32 {
        println!("Image needs more pixels !");
        exit(1);
    }

    let mut img_buf = img.as_mut_rgb8().unwrap();
    let mut it = ImageIterator::new(img_buf);

    // Add header (payload size)
    let mut size_bytes = vec![];
    size_bytes.write_u32::<BigEndian>(bytes_to_hide.len() as u32).unwrap();
    println!("{:?}", size_bytes);
    write_bytes(img_buf, &mut it, &size_bytes);

    // Add actual payload
    write_bytes(img_buf, &mut it, &bytes_to_hide);
    println!("wrote {:?}", bytes_to_hide);

    img_buf.save(&Path::new(path_output)).unwrap();
}

fn write_bytes(img_buf: &mut RgbImage, it: &mut ImageIterator, bytes_to_hide: &Vec<u8>) {
    for byte in bytes_to_hide {
        for bit_index in 0..8 {
            let bit_to_hide = ((*byte >> bit_index) & 1) as u8;
            let next = it.next().unwrap();
            let mut pixel = img_buf.get_pixel_mut(next.x, next.y);
            let chan_index = next.channel as usize;

            pixel.data[chan_index] = pixel.data[chan_index] & 0xFE | bit_to_hide;
        }
    }
}

fn read_bytes(img_buf: &RgbImage, it: &mut ImageIterator, bytes: &mut Vec<u8>) {
    for byte in bytes {
        for bit_index in 0..8 {
            let next = it.next().unwrap();
            let pixel = img_buf.get_pixel(next.x, next.y);
            let chan_index = next.channel as usize;

            if pixel.data[chan_index] & (1 << 0) == 1 {
                // Set nth bit to 1
                *byte |= 1 << bit_index;
            }
            // else bit is 0 and is already set
        }
    }
}

fn unsteg(path: &str, bytes: &mut Vec<u8>) {
    let img = image::open(&Path::new(path)).unwrap();
    let img_buf = img.as_rgb8().unwrap();
    let mut it = ImageIterator::new(img_buf);

    // Get payload header (payload size)
    let mut size_bytes = vec![0; 4];
    read_bytes(img_buf, &mut it, &mut size_bytes);

    let mut rdr = Cursor::new(size_bytes);
    let size = rdr.read_u32::<BigEndian>().unwrap() as usize;
    println!("Header contains size {:?}", size);
    *bytes = vec![0; size];
    read_bytes(img_buf, &mut it, bytes);
    println!("read bytes {:?}", bytes);
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
