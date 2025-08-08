use crate::Compressable;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::io::{Read, Write};

#[derive(Debug)]
pub enum DeflateError {
    IO(std::io::Error),
    InvalidCheckBytes(std::array::TryFromSliceError),
}

#[derive(Debug, Clone)]
pub struct DeflateStream {
    compression_method: u8,
    flags: u8,
    data_blocks: Box<[u8]>,
    check_value: u32,
}

impl Compressable for DeflateStream {
    type Error = DeflateError;
    fn try_compress(&self) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let self_vec: Vec<u8> = self.clone().into();

        let mut decompressed_data = Vec::new();
        {
            let mut decoder = ZlibDecoder::new(&*self_vec);
            decoder
                .read_to_end(&mut decompressed_data)
                .map_err(DeflateError::IO)?;
        }

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder
            .write_all(&decompressed_data)
            .map_err(DeflateError::IO)?;
        let compressed = encoder.finish().map_err(DeflateError::IO)?;

        println!("{} {}", decompressed_data.len(), compressed.len());
        DeflateStream::try_create(compressed)
    }
}

impl Into<Vec<u8>> for DeflateStream {
    fn into(self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        vec.push(self.compression_method);
        vec.push(self.flags);
        vec.extend(self.data_blocks.iter());
        vec.extend(self.check_value.to_be_bytes());

        vec
    }
}

impl DeflateStream {
    pub fn try_create<I>(data: I) -> Result<Self, DeflateError>
    where
        I: IntoIterator<Item = u8>,
    {
        let vec = data.into_iter().collect::<Vec<u8>>();

        let compression_method = vec[0];
        let flags = vec[1];

        let check_value_bytes: [u8; 4] = vec[vec.len() - 4..]
            .try_into()
            .map_err(DeflateError::InvalidCheckBytes)?;
        let data_blocks: Box<[u8]> = vec[2..vec.len() - 4].into();
        let check_value = u32::from_be_bytes(check_value_bytes);

        Ok(DeflateStream {
            compression_method,
            flags,
            data_blocks,
            check_value,
        })
    }
}
