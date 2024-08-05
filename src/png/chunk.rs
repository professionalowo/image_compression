use super::{PngError, PngResult};

#[derive(Debug, Clone)]
pub struct Chunk {
    pub length: u32,
    pub chunk_type: [u8; 4],
    pub chunk_data: Box<[u8]>,
    pub crc: u32,
}

impl Into<Vec<u8>> for Chunk {
    fn into(self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        vec.extend(self.length.to_be_bytes());
        vec.extend(self.chunk_type);
        vec.extend(self.chunk_data.iter());
        vec.extend(self.crc.to_be_bytes());

        vec
    }
}

impl Chunk {
    pub fn set_data(&mut self, data: Box<[u8]>) {
        self.chunk_data = data;
    }

    pub fn try_create<I>(length: u32, data: I) -> PngResult<Self>
    where
        I: IntoIterator<Item = u8>,
    {
        let bytes: Vec<u8> = data
            .into_iter()
            .take((length + 8).try_into().unwrap())
            .collect();

        let chunk_type: [u8; 4] = match bytes[0..4].try_into() {
            Err(_) => return Err(PngError::new("not enough data")),
            Ok(b) => b,
        };

        let chunk_data: Box<[u8]> = bytes[4..bytes.len() - 4].into();

        let crc_bytes: [u8; 4] = match bytes[bytes.len() - 4..].try_into() {
            Err(_) => return Err(PngError::new("not enough data")),
            Ok(b) => b,
        };

        let crc = u32::from_be_bytes(crc_bytes);

        let data_length = length - 8;

        let chunk = Self {
            length: data_length,
            chunk_type,
            chunk_data,
            crc,
        };
        Ok(chunk)
    }

    pub fn is_ancillary(&self) -> bool {
        Chunk::is_bit_set(&self.chunk_type[0], 5)
    }

    pub fn is_private(&self) -> bool {
        Chunk::is_bit_set(&self.chunk_type[1], 5)
    }

    pub fn is_reserved(&self) -> bool {
        Chunk::is_bit_set(&self.chunk_type[2], 5)
    }

    pub fn is_save_to_copy(&self) -> bool {
        Chunk::is_bit_set(&self.chunk_type[3], 5)
    }

    fn is_bit_set(data: &u8, index: usize) -> bool {
        let mask = 1 << index;
        (mask & data) > 0
    }
}
