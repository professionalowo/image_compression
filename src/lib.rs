mod crc;
mod deflate;
pub mod png;

pub trait Compressable {
    type Error;
    fn try_compress(&self) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
