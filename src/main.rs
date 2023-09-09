use std::{env, path::Path, fs::File, io::{Read, Write}, borrow::Borrow};
use bmp::{Image, px, Pixel};
mod png;
use png::png_to_intermediate;

pub struct PixelRGB(u8, u8, u8);
pub struct PixelRGBA(u8, u8, u8, u8);

pub enum Pixels {
    RGB(Vec<PixelRGB>),
    RGBA(Vec<PixelRGBA>)
}

pub struct IntermediateImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Pixels
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = args[1].clone();
    let path = Path::new(&filename);
    if !path.is_file() {
        panic!("File not found");
    }
    let ext;
    if let Some(e) = path.extension() {
        ext = e.clone().to_string_lossy();
    } else {
        panic!("No extension");
    }
    let mut img: IntermediateImage;
    match ext.borrow() {
        "png" => {
            img = png_to_intermediate(path);
        }
        filetype => panic!("Unsupported file type {}", filetype)
    }
    let mut bmp = Image::new(img.width, img.height);
    let px;
    if let Pixels::RGB(ref pxs) = img.pixels {
        px = pxs.clone();
    } else {
        panic!("huh");
    }
    println!("should be of size {} but is {}", img.height*img.width, px.len());
    for (x,y) in bmp.coordinates() {
        let idx = (y*img.width)+x;
        let (r,g,b);
        let prgb = &px[idx as usize];
        r = prgb.0;
        g = prgb.1;
        b = prgb.2;

        bmp.set_pixel(x, y, px!(r,g,b));
    }
    bmp.save("out.bmp").unwrap();
}
