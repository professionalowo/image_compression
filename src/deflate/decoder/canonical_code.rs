use std::{collections::VecDeque, io::Read};

use super::{DecoderResult, DeflateError};
struct CanonicalCode {
    symbol_code_bits: Vec<u64>,
    symbol_values: Vec<u64>,
}

impl CanonicalCode {
    pub fn try_new<I>(code_lengths_iter: I) -> DecoderResult<Self>
    where
        I: IntoIterator<Item = u64>,
    {
        let code_lengths: Vec<u64> = code_lengths_iter.into_iter().collect();

        if !code_lengths.iter().all(|x| *x > 0) {
            return Err(DeflateError::default());
        };

        let mut symbol_code_bits_vec: Vec<u64> = Vec::new();
        let mut symbol_value_vec: Vec<u64> = Vec::new();
        let mut num_symbols_allocated = 0;
        let mut next_code = 0;
        for code_length in 1..MAX_CODE_LENGTH {
            next_code <<= 1;
            let start_bit = 1 << code_length;

            for symbol in 0..code_lengths.len() {
                if code_lengths[symbol as usize] != (code_length as u64) {
                    continue;
                };

                if next_code >= start_bit {
                    return Err(DeflateError(
                        "This canonical code produces an over-full Huffman code tree".into(),
                    ));
                };
                symbol_code_bits_vec[num_symbols_allocated] = start_bit | next_code;
                symbol_value_vec[num_symbols_allocated] = symbol as u64;
                num_symbols_allocated += 1;
                next_code += 1;
            }
        }

        if next_code != 1 << MAX_CODE_LENGTH {
            return Err(DeflateError(
                "This canonical code produces an under-full Huffman code tree".into(),
            ));
        }

        Ok(Self {
            symbol_code_bits: symbol_code_bits_vec,
            symbol_values: symbol_value_vec,
        })
    }

    pub fn decode_next_symbol<I>(&self, input: &mut BitIter) -> DecoderResult<u64>
    where
        I: IntoIterator<Item = u8>,
    {
        let mut code_bits = 1;
        loop {
            code_bits = code_bits << 1 | input.read_uint(1)?;
            match self.symbol_code_bits.binary_search(&(code_bits as u64)) {
                Ok(s) => return Ok(self.symbol_values[s]),
                Err(_) => return Err(DeflateError("unknown huffman code".into())),
            }
        }
    }
}

#[derive(Debug)]
struct BitIter {
    current_byte: Option<u8>,
    remaining: u8,
    inner: VecDeque<u8>,
}
impl BitIter {
    pub fn new<I>(inner: I) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        let inner_vec: VecDeque<u8> = inner.into_iter().collect();

        Self {
            current_byte: Some(0),
            remaining: 0,
            inner: inner_vec,
        }
    }

    fn get_bit_position(&self) -> u8 {
        (8 - self.remaining) % 8
    }

    fn read_next_maybe(&mut self) -> Option<u8> {
        if self.current_byte == None {
            return None;
        };
        if self.remaining == 0 {
            self.current_byte = self.inner.pop_front().map(|b| b.reverse_bits());
            match self.current_byte {
                None => return None,
                Some(_) => self.remaining = 8,
            }
        };
        self.remaining -= 1;

        self.current_byte
            .map(|byte| (byte >> (7 - self.remaining)) & 1)
    }

    pub fn read_uint(&mut self, num_bits: u8) -> DecoderResult<u8> {
        if num_bits > 31 {
            return Err(DeflateError("Number of bits out of range".into()));
        };
        let mut result = 0;
        for i in 0..num_bits {
            let bit = match self.read_next_maybe() {
                Some(b) => b,
                None => return Err(DeflateError("Unexpected end of stream".into())),
            };
            result |= bit << i;
        }
        Ok(result.reverse_bits())
    }
}

const MAX_CODE_LENGTH: u8 = 15;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bit_iter_test() {
        let vec: Vec<u8> = vec![10];

        let mut iter: BitIter = BitIter::new(vec);
        let byte = iter.read_uint(8);
        assert_eq!(byte.unwrap(), 10)
    }
}
