extern crate image;
extern crate byteorder;
extern crate clap;
extern crate rand;


use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::io::Cursor;
use std::fmt;
use std::error;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use clap::{Arg, App, ArgMatches, SubCommand, AppSettings};
use rand::{thread_rng, Rng};
use image::*;

fn main() {
    let matches = App::new("rust-alsb")
                      .version("0.0.1")
                      .author("Justin Duplessis <jdupl@linux.com>")
                      .about("Simple stetanography with an Advanced Least Significant Bit \
                              algorithm. This software should NOT be considered SECURE as it is \
                              wrote for educational purposes.")
                      .setting(AppSettings::SubcommandRequired)
                      .subcommand(SubCommand::with_name("steg")
                                      .about("Hide some data in an image file. Outputs PNG.")
                                      .arg(Arg::with_name("input")
                                               .help("File to get public data from.")
                                               .required(true)
                                               .index(1))
                                      .arg(Arg::with_name("to_hide")
                                               .help("File to hide.")
                                               .required(true)
                                               .index(2))
                                      .arg(Arg::with_name("output")
                                               .help("Output path of stegged file. Extension \
                                                      should be '.png'")
                                               .required(true)
                                               .index(3)))
                      .subcommand(SubCommand::with_name("unsteg")
                                      .about("Reveal some data from a PNG file.")
                                      .arg(Arg::with_name("input")
                                               .help("Sets the input file to use.")
                                               .required(true)
                                               .index(1))
                                      .arg(Arg::with_name("output")
                                               .help("Sets the output file to use.")
                                               .required(true)
                                               .index(2)))
                      .get_matches();
                    let _ = wrap_user(matches);


}

fn wrap_user(matches: ArgMatches) -> Result<(), Error> {
    match matches.subcommand() {
        ("steg", Some(v)) => {
            try!(steg_wrap(v.value_of("input").unwrap(),
                 v.value_of("output").unwrap(),
                 v.value_of("to_hide").unwrap()))
        }
        ("unsteg", Some(v)) => {
            try!(unsteg(v.value_of("input").unwrap(), v.value_of("output").unwrap()))
        }
        _ => panic!("No subcommand provided by user !"),
    }
    Ok(())
}

fn steg_wrap(path_input: &str, path_output: &str, path_input_hide: &str) -> Result<(), Error> {
    let mut bytes_to_hide = Vec::new();
    let mut fin = File::open(path_input_hide).unwrap();
    let _ = fin.read_to_end(&mut bytes_to_hide);

    let mut img = image::open(&Path::new(path_input)).unwrap();
    try!(steg(&mut bytes_to_hide, &mut img));

    let ref mut fout = File::create(&Path::new(path_output)).unwrap();
    img.save(fout, ImageFormat::PNG).unwrap();
    Ok(())
}

fn steg(bytes_to_hide: &Vec<u8>, img: &mut DynamicImage) -> Result<(), Error> {
    let (dim_x, dim_y) = img.dimensions();

    if dim_x * dim_y * 3 / 8 <= bytes_to_hide.len() as u32 {
        return Err(Error::NotEnoughPixels);
    }
    let mut img_buf = img.as_mut_rgb8().unwrap();
    let mut it = ImageIterator::new(img_buf);

    let mut random_bytes = [0u8; 128]; // TODO Get random size
    thread_rng().fill_bytes(&mut random_bytes);

    write_payload_with_header(&random_bytes.to_vec(), img_buf, &mut it);
    write_payload_with_header(&bytes_to_hide, img_buf, &mut it);

    Ok(())
}

fn write_payload_with_header(payload: &Vec<u8>, img_buf: &mut RgbImage, it: &mut ImageIterator) {
    // Add header (payload size)
    let mut size_bytes = vec![];
    size_bytes.write_u32::<BigEndian>(payload.len() as u32).unwrap();
    write_bytes(img_buf, it, &size_bytes);

    // Add actual payload
    write_bytes(img_buf, it, &payload);
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

fn get_next_payload(img_buf: &RgbImage, it: &mut ImageIterator) -> Vec<u8> {
    // Get payload header (payload size)
    let mut size_bytes = vec![0; 4];
    read_bytes(img_buf, it, &mut size_bytes);

    let mut rdr = Cursor::new(size_bytes);
    let size = rdr.read_u32::<BigEndian>().unwrap() as usize;

    // let (dim_x, dim_y) = img.dimensions();
    // if dim_x * dim_y * 3 / 8 <= size as u32 {
    //     println!("Input file has an invalid payload size in header.");
    //     println!("Image does not have enough pixels !");
    //     // return Err(Error::InvalidFormat);
    //     panic!("") // TODO FIXME
    // }

    let mut bytes = vec![0; size]; // create output buffer
    read_bytes(img_buf, it, &mut bytes);

    return bytes;
}

fn unsteg_bytes(img: DynamicImage) -> Vec<u8> {
    let img_buf = img.as_rgb8().unwrap();
    let mut it = ImageIterator::new(img_buf);

    // Get random data
    let xored: Vec<u8> = get_next_payload(img_buf, &mut it);
    // TODO unxor

    return get_next_payload(img_buf, &mut it);
}

fn unsteg(path_input: &str, path_output: &str) -> Result<(), Error> {
    let img = image::open(&Path::new(path_input)).unwrap();
    // let img_buf = img.as_rgb8().unwrap();

    let mut bytes = unsteg_bytes(img);
    println!("Saving unstegged bytes to {}", path_output);
    let mut f = File::create(path_output).unwrap();
    f.write_all(&mut bytes).unwrap();
    Ok(())
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
            x: self.curr_x,
            y: self.curr_y,
            channel: self.curr_channel,
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

#[derive(Debug)]
pub enum Error {
    NotEnoughPixels,
    InvalidFormat,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NotEnoughPixels  => write!(f, "Not enought pixels to hide message."),
            Error::InvalidFormat  => write!(f, "Invalid image format."),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::NotEnoughPixels  => None,
            Error::InvalidFormat  => None,
        }
    }

    fn description(&self) -> &str {
        match *self {
            Error::NotEnoughPixels  => "Input image error",
            Error::InvalidFormat  => "Input image error",
        }
    }
}

#[cfg(test)]
mod test {
    use super::{steg, unsteg_bytes};
    use image::*;
    use std::path::Path;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_unwrapped_steg() {
        // Generate test input image
        let img_in = ImageBuffer::from_fn(512, 512, |x, y| {
            if x % 2 == 0 || y % 2 == 0 {
                Rgb([0u8, 0u8, 0u8])
            } else {
                Rgb([255u8, 255u8, 255u8])
            }
        });
        // TODO convert buffer to DynamicImage without IO ?
        let _ = img_in.save(&Path::new("test_in.png")).unwrap();
        let mut img = open(&Path::new("test_in.png")).unwrap();

        let mut secret_bytes = [0u8; 128];
        thread_rng().fill_bytes(&mut secret_bytes);

        steg(&secret_bytes.to_vec(), &mut img).unwrap();
        let found_bytes = unsteg_bytes(img);

        assert!(found_bytes == secret_bytes.to_vec());
    }

    #[test]
    #[should_panic]
    fn test_not_enough_pixels() {
        // Generate test input image
        let img_in = ImageBuffer::from_fn(12, 12, |x, y| {
            if x % 2 == 0 || y % 2 == 0 {
                Rgb([0u8, 0u8, 0u8])
            } else {
                Rgb([255u8, 255u8, 255u8])
            }
        });
        // TODO convert buffer to DynamicImage without IO ?
        let _ = img_in.save(&Path::new("test_in.png")).unwrap();
        let mut img = open(&Path::new("test_in.png")).unwrap();

        let mut secret_bytes = [0u8; 128];
        thread_rng().fill_bytes(&mut secret_bytes);

        steg(&secret_bytes.to_vec(), &mut img).unwrap();
        let found_bytes = unsteg_bytes(img);

        assert!(found_bytes == secret_bytes.to_vec());
    }
}
