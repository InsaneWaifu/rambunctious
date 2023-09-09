use core::panic;
use std::{path::Path, fs::File, io::{BufReader, Read, Seek}};
use inflate::inflate_bytes_zlib;
use crate::{IntermediateImage, PixelRGB};

#[derive(Default)]
struct PNGInfo {
    width: u32,
    height: u32,
    bitdepth: u8,
    color_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_adam7: bool
}

impl PNGInfo {
    fn from_idhr(&mut self, idhr: &[u8]) {
        self.width = u32::from_be_bytes(idhr[0..4].try_into().unwrap());
        self.height = u32::from_be_bytes(idhr[4..8].try_into().unwrap());
        self.bitdepth = idhr[8];
        self.color_type = idhr[9];
        self.compression_method = idhr[10];
        self.filter_method = idhr[11];
        self.interlace_adam7 = idhr[12] == 1;
        assert!(!self.interlace_adam7);
    }

    fn get_bpp(&self) -> u8 {
        match self.color_type {
            0 => self.bitdepth,
            2 => self.bitdepth*3,
            3 => 8,
            4 => self.bitdepth*2,
            6 => self.bitdepth*4,
            _ => panic!("Unsupported color type")
        }
    }
}

pub fn png_to_intermediate(path: &Path) -> IntermediateImage {
    let png = File::open(path).unwrap();
    let mut reader = BufReader::new(png);
    let mut header: [u8;8] = [0;8];
    let correct_header: [u8;8] = [0x89,0x50,0x4E,0x47,0x0D,0x0A,0x1A,0x0A];
    reader.read_exact(&mut header).unwrap();
    assert_eq!(correct_header, header);
    
    let mut info = PNGInfo::default();
    let mut palette: Vec<(u8, u8, u8)> = vec![];
    let mut fulldata: Vec<u8> = vec![];
    loop {
        let mut length: [u8;4] = [0;4];
        let mut ctype: [u8;4] = [0;4];
        reader.read_exact(&mut length).unwrap();
        reader.read_exact(&mut ctype).unwrap();
        let len = u32::from_be_bytes(length);
        let cty: &str = &*String::from_utf8(ctype.to_vec()).unwrap();
        let mut data: Vec<u8> = vec![0;len as usize];
        let mut crc: [u8;4] = [0;4];
        reader.read_exact(&mut data).unwrap();
        reader.read_exact(&mut crc).unwrap();

        match cty {
            "IHDR" => info.from_idhr(&data[..]),
            "PLTE" => {
                let samples = len / 3;
                for i in 0..samples {
                    let r = data[(i*3) as usize];
                    let g = data[(i*3+1) as usize];
                    let b = data[(i*3+2) as usize];
                    palette.push((r,g,b));
                }
            },
            "IDAT" => {
                fulldata.extend(&data[..]);
            },
            "IEND" => {
                break;
            },
            _ => () // ignore unrecognised chunk
        }
    }
    let realdata = inflate_bytes_zlib(&fulldata).unwrap();
    let mut br = BufReader::new(&realdata[..]);
    let mut px = vec![];
    assert_eq!(info.bitdepth, 8);
    // read scanlines
    for y in 0..info.height {
        let mut filtype: [u8;1] = [0;1];
        br.read_exact(&mut filtype).unwrap();
        let process;
        match filtype[0] {
            _ => process = |x: u8|x,
            //_ => panic!("Unsupported filter type {}", filtype[0])
        }
        let mut scanline: Vec<u8> = vec![0;(info.width * (info.get_bpp() as u32/8)) as usize];
        br.read_exact(&mut scanline).unwrap();
        for x in (0..(info.width*(info.get_bpp() as u32/8))).step_by(info.get_bpp() as usize/8) {
            match info.color_type {
                0 | 4 => {
                    let val = process(scanline[x as usize]);
                    px.push(PixelRGB(val, val, val));
                },
                2 | 6 => {
                    let r = scanline[x as usize];
                    let g = scanline[(x+1) as usize];
                    let b = scanline[(x+2) as usize];
                    px.push(PixelRGB(r,g,b));
                },
                3 => {
                    let idx = scanline[x as usize];
                    let (r,g,b) = palette[idx as usize];
                    px.push(PixelRGB(r,g,b));
                },
                _ => panic!("Unsupported color type")
            };
        }
    }
    IntermediateImage { width: info.width, height: info.height, pixels: crate::Pixels::RGB(px) }
}