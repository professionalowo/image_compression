use std::{
    fs,
    io::{Read, Write},
};

use png::PngImage;

mod crc;
mod deflate;
pub mod png;

pub trait Compressable {
    type Error;
    fn try_compress(&self) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        args.push("./Screenshot_2024-07-06_140840.png".into());
        args.push("./o.png".into())
    }

    let input = &args[1];
    let output = &args[2];

    let bytes = fs::read(&input).expect("could not read file");

    let png = PngImage::try_create(bytes).expect("could not create png");

    let compressed = png.try_compress().unwrap();
    let comp_vec: Vec<u8> = compressed.clone().into();
    let mut outfile = fs::File::create(&output).unwrap();
    outfile.write_all(&comp_vec).unwrap();

    println!("{:#?}", &comp_vec.len());
}
