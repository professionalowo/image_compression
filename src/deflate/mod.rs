use crate::Compressable;

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
        todo!();
    }
}

pub struct DeflateError {}

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

        let check_value_bytes: [u8; 4] = match vec[vec.len() - 4..].try_into() {
            Ok(b) => b,
            Err(_) => return Err(DeflateError {}),
        };
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
