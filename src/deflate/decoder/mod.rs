//see: https://github.com/nayuki/Simple-DEFLATE-decompressor/blob/master/java/src
use std::io::Error;

use super::DeflateError;
use byte_window::ByteWindow;

mod byte_window;
mod canonical_code;

type DecoderResult<T> = Result<T, DeflateError>;

#[derive(Debug)]
pub struct Decoder {
    data: Box<[u8]>,
    byte_window: ByteWindow<32768>,
}

impl Decoder {
    pub fn new<I>(data_source: I) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        let byte_window = ByteWindow::<32768>::new();
        let data: Box<[u8]> = data_source.into_iter().collect();
        Self { data, byte_window }
    }

    pub fn decode(&mut self) -> DecoderResult<Box<[u8]>> {
        Ok(Box::new([0]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_constructor() {
        let vec = b"Hello World Hello World".to_vec();

        let mut decoder = Decoder::new(vec);

        let decompressed = decoder.decode();

        assert!(decompressed.is_ok())
    }
}
