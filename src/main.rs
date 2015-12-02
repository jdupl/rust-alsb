extern crate image;
extern crate bigwise;

use std::path::Path;

use image::*;

fn main() {
    let mut img = image::open(&Path::new("test.jpg")).unwrap();
    println!("dimensions {:?}", img.dimensions());

    let mut img_buf = img.as_mut_rgb8().unwrap();

    for (x, y, pixel) in img_buf.enumerate_pixels_mut() {
        // Change pixel
        pixel.invert();
    }

    img_buf.save(&Path::new(&format!("{}.png", "test.out"))).unwrap();
}
