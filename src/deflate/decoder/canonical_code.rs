use std::io::Read;

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

    pub fn decode_next_symbol<I>(&self, input: I) -> DecoderResult<u64>
    where
        I: IntoIterator<Item = u64>,
    {
        todo!("read bit by bit");
        let mut iter = input.into_iter();
        let mut code_bits = 1;
        loop {
            code_bits = code_bits << 1 | iter.next().unwrap();
            match self.symbol_code_bits.binary_search(&code_bits) {
                Ok(s) => return Ok(self.symbol_values[s]),
                Err(_) => return Err(DeflateError("unknown huffman code".into())),
            }
        }
    }
}

const MAX_CODE_LENGTH: u8 = 15;
