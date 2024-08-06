use std::io::Error;

use crate::deflate::Compression;

#[derive(Debug)]
pub struct Encoder {
    compression: Compression,
    data: Box<[u8]>,
}

impl Encoder {
    pub fn new<I>(data_source: I, compression: Compression) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        let data: Box<[u8]> = data_source.into_iter().collect();
        Self { compression, data }
    }

    pub fn encode(&self) -> Result<Box<[u8]>, Error> {
        Ok(Box::new([0]))
    }
}
