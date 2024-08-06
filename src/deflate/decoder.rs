//see: https://github.com/nayuki/Simple-DEFLATE-decompressor/blob/master/java/src/ByteHistory.java
use std::{io::Error, ops::Range};

#[derive(Debug)]
pub struct Decoder {
    data: Box<[u8]>,
}

impl Decoder {
    pub fn new<I>(data_source: I) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        let data: Box<[u8]> = data_source.into_iter().collect();
        Self { data }
    }

    pub fn decode(&self) -> Result<Box<[u8]>, Error> {
        let window = ByteWindow::<{ 32 * 1024 }>::new();
        Ok(Box::new([0]))
    }
}

struct ByteWindow<const COUNT: usize> {
    written: u64,
    index: usize,
    data: [u8; COUNT],
}

impl<const COUNT: usize> ByteWindow<COUNT> {
    pub const fn new() -> Self {
        Self {
            written: 0,
            index: 0,
            data: [0; COUNT],
        }
    }

    pub fn append(&mut self, byte: u8) {
        self.data[self.index] = byte;
        self.index = (self.index + 1) % COUNT;

        if self.written < COUNT.try_into().unwrap() {
            self.written += 1;
        }
    }

    pub fn copy(&mut self, distance: usize, length: usize, buf: &mut [u8]) {
        let mut read_index: usize = ((self.index) - distance - COUNT) % (COUNT);
        let range: Range<usize> = 0..length;
        for i in range {
            let byte: u8 = self.data[read_index];
            read_index = (read_index + 1) % COUNT;
            buf[i] = byte;
            self.append(byte)
        }
    }
}
