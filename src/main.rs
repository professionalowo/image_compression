use std::{
    env, fs,
    io::{Error, Write},
};

use png::PngImage;

pub mod png;

pub trait Compressable {
    fn compress(&self) -> Self;
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

    let compressed = png.compress();

    let compressed_size = compressed.size();

    let bytes_out: Vec<u8> = compressed.into();
    outfile.write_all(&bytes_out)?;

    println!("Compressed: {} -> {}", png.size(), compressed_size);

    Ok(())
}
