use std::{
    env, fs,
    io::{Error, Write},
};

use png::PngImage;

mod deflate;
mod crc;
pub mod png;

pub trait Compressable {
    type Error;
    fn try_compress(&self) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("{}", "Usage: 'image_compression <input> <output>'");
    }
    println!("{:#?}", args);

    let input = &args[1];
    let output = &args[2];

    let bytes_in = fs::read(input)?;

    let png = PngImage::try_create(bytes_in).unwrap();

    println!("{:X?}", &png.file_signature);
    println!("{:#?}", &png.header);

    let mut outfile = fs::File::create(output)?;

    let compressed = png.try_compress().unwrap();

    let compressed_size = compressed.size();

    let bytes_out: Vec<u8> = compressed.into();
    outfile.write_all(&bytes_out)?;

    let chunk = &png.get_data()[2];

    println!(
        "{:X?}\n{},{},{},{}",
        chunk,
        chunk.is_ancillary(),
        chunk.is_private(),
        chunk.is_reserved(),
        chunk.is_save_to_copy()
    );

    println!("Compressed: {} -> {}", png.size(), compressed_size);

    Ok(())
}
