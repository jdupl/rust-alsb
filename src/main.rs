extern crate image;

use std::path::Path;
use std::process::exit;

use image::*;

fn main() {
    let bytes = [12, 41, 123, 4, 75];

    let mut img = image::open(&Path::new("test.jpg")).unwrap();
    let (dim_x, dim_y) = img.dimensions();

    if dim_x * dim_y * 3 / 8  <=  bytes.len() as u32 {
        println!("Image needs more pixels !");
        exit(1);
    }

    let mut img_buf = img.as_mut_rgb8().unwrap();
    let mut it = ImageIterator::new(dim_x, dim_y);

    for byte in bytes.iter() {
        for bit in 0..7 {
            let bit_to_hide = ((byte >> bit) & 1) as u8;

            let next = it.next().unwrap();
            let mut pixel = img_buf.get_pixel_mut(next.x, next.y);

            // println!("Hiding value {} in channel {}", bit_to_hide, next.channel);
            // println!("{:?}", pixel);
            pixel.data[next.channel as usize] = pixel.data[next.channel as usize] & 0xFE | bit_to_hide;
            // println!("{:?}", pixel);
        }
    }

    img_buf.save(&Path::new(&format!("{}.png", "test.out"))).unwrap();
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
    pub fn new(max_x: u32, max_y: u32) -> Self {
        ImageIterator {
            max_x: max_x,
            max_y: max_y,
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
