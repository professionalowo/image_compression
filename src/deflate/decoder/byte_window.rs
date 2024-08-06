use super::{DecoderResult, DeflateError};
use std::ops::Range;
#[derive(Debug)]
pub struct ByteWindow<const COUNT: usize> {
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

    pub fn append(&mut self, byte: u8) -> DecoderResult<()> {
        if self.index > COUNT {
            return Err(DeflateError("Unreachable state".into()));
        }

        self.data[self.index] = byte;
        self.index = (self.index + 1) % COUNT;

        if self.written < COUNT.try_into().unwrap() {
            self.written += 1;
        };
        Ok(())
    }

    pub fn copy(&mut self, distance: usize, length: usize, buf: &mut [u8]) -> DecoderResult<()> {
        if distance < 1 || distance > length {
            return Err(DeflateError::default());
        }

        let mut read_index: usize = ((self.index) - distance - COUNT) % (COUNT);
        let range: Range<usize> = 0..length;
        for i in range {
            let byte: u8 = self.data[read_index];
            read_index = (read_index + 1) % COUNT;
            buf[i] = byte;
            self.append(byte)?;
        }
        Ok(())
    }
}
