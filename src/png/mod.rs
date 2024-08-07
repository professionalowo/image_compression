use chunk::Chunk;
use deflate::DeflateStream;
use ihdrheader::IhdrHeader;

use crate::Compressable;
mod chunk;
mod crc;
mod deflate;
mod ihdrheader;
type PngResult<T> = std::result::Result<T, PngError>;
#[derive(Debug, Clone)]
pub struct PngError {
    pub reason: String,
}
impl PngError {
    fn new<I>(reason: I) -> Self
    where
        I: Into<String>,
    {
        PngError {
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PngImage {
    pub header: IhdrHeader,
    pub file_signature: [u8; 8],
    data: Box<[Chunk]>,
}

impl PngImage {
    pub fn try_create<I>(data: I) -> PngResult<Self>
    where
        I: IntoIterator<Item = u8>,
    {
        let mut iter = data.into_iter();
        let mut file_signature: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        for i in 0..8 {
            if let Some(byte) = iter.next() {
                file_signature[i] = byte.clone();
            }
        }
        assert_eq!(
            file_signature,
            [137, 80, 78, 71, 13, 10, 26, 10],
            "file signature is wrong"
        );
        let chunks = chunk(iter)?;

        let header: IhdrHeader = chunks[0].clone().try_into()?;
        Ok(PngImage {
            header,
            file_signature,
            data: chunks.into(),
        })
    }

    pub fn get_data(&self) -> &[Chunk] {
        &self.data
    }

    pub fn size(&self) -> usize {
        self.file_signature.len()
            + self
                .data
                .iter()
                .map(|chunk| -> usize { chunk.length.try_into().unwrap() })
                .sum::<usize>()
    }
}

fn chunk<I>(data: I) -> PngResult<Vec<Chunk>>
where
    I: IntoIterator<Item = u8>,
{
    let data_vec: Vec<u8> = data.into_iter().collect();

    let mut chunks: Vec<Chunk> = Vec::new();
    let mut iter = data_vec.iter();

    while let Some(data) = iter.next() {
        let mut len_bytes: Vec<u8> = Vec::new();
        len_bytes.push(data.clone());
        len_bytes.extend(iter.next());
        len_bytes.extend(iter.next());
        len_bytes.extend(iter.next());

        let length_bytes: [u8; 4] = match len_bytes.try_into() {
            Ok(b) => b,
            Err(_) => return Err(PngError::new("not enough data")),
        };

        let length = u32::from_be_bytes(length_bytes) + 4 + 4;

        let chunk = Chunk::try_create(
            length,
            iter.clone()
                .take(length.try_into().unwrap())
                .map(|b| b.clone())
                .collect::<Vec<u8>>(),
        )?;

        chunks.push(chunk);

        for _ in 0..length {
            iter.next();
        }
    }

    Ok(chunks)
}

impl Compressable for PngImage {
    type Error = PngError;
    fn try_compress(&self) -> PngResult<Self> {
        let mut out = self.clone();
        let data = self.data.clone();

        //strip auxilliary chunks
        let mut only_critical: Vec<Chunk> = data
            .iter()
            .filter_map(|chunk| (!chunk.is_ancillary()).then_some(chunk.clone()))
            .collect();

        let only_critical_len = only_critical.len();

        let only_idat: &mut [Chunk] = only_critical[1..only_critical_len - 1].as_mut();

        let idat_stream_data = only_idat
            .iter()
            .flat_map(|chunk| -> Vec<u8> { chunk.chunk_data.to_vec() })
            .collect::<Vec<u8>>();

        let deflate = match DeflateStream::try_create(idat_stream_data.clone()) {
            Ok(stream) => stream,
            Err(_) => return Err(PngError::new("could not create inflate stream")),
        };

        let compressed_deflate = match deflate.try_compress() {
            Ok(comp) => comp,
            Err(_) => return Err(PngError::new("could not compress data stream")),
        };
        const IDAT_SIZE: usize = 65_524;
        let mut out_chunks: Vec<Chunk> = Vec::new();
        let ihdrheader: Chunk = out.data[0].clone();
        out_chunks.push(ihdrheader);
        let deflate_bytes: Vec<u8> = compressed_deflate.into();

        let crc32 = crc::Crc32::new();

        let chunked: Vec<Chunk> = deflate_bytes
            .chunks(IDAT_SIZE)
            .map(|chunk| -> Chunk {
                let length: u32 = chunk.len().try_into().unwrap();
                let chunk_type = IDAT_TYPE;
                let chunk_data: Box<[u8]> = chunk.into();

                let mut bytes: Vec<u8> = Vec::new();
                bytes.extend(chunk_type);
                bytes.extend(chunk_data.iter());

                let crc = crc32.crc(&bytes);
                Chunk {
                    length,
                    chunk_type,
                    chunk_data,
                    crc,
                }
            })
            .collect();
        out_chunks.extend(chunked);
        out_chunks.push(out.data[out.data.len() - 1].clone());
        out_chunks.shrink_to_fit();
        out.data = out_chunks.into();

        Ok(out)
    }
}

impl Into<Vec<u8>> for PngImage {
    fn into(self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        vec.extend(self.file_signature);
        vec.extend(
            self.data
                .iter()
                .flat_map(|c| -> Vec<u8> { Chunk::into(c.clone()) })
                .collect::<Vec<u8>>(),
        );

        vec
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum ColorType {
    GRAYSCALE = 0,
    RGB = 2,
    PLTE = 3,
    GRAYSCALEALPHA = 4,
    RGBA = 6,
}

impl TryFrom<u8> for ColorType {
    type Error = PngError;
    fn try_from(value: u8) -> PngResult<Self> {
        match value {
            0 => Ok(Self::GRAYSCALE),
            2 => Ok(Self::RGB),
            3 => Ok(Self::PLTE),
            4 => Ok(Self::GRAYSCALEALPHA),
            6 => Ok(Self::RGBA),
            _ => Err(PngError::new("unknown color type")),
        }
    }
}

impl Into<u8> for ColorType {
    fn into(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone)]
pub enum CompressionMethod {
    DEFLATE = 0,
}
#[derive(Debug, Clone)]
pub enum FilterMethod {
    NONE,
    SUB,
    UP,
    AVERAGE,
    PAETH,
}

impl TryFrom<u8> for FilterMethod {
    type Error = PngError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::NONE),
            1 => Ok(Self::SUB),
            2 => Ok(Self::UP),
            3 => Ok(Self::AVERAGE),
            4 => Ok(Self::PAETH),

            _ => Err(PngError::new("unknown filter type")),
        }
    }
}
#[derive(Debug, Clone)]
pub enum InterlaceMethod {
    NONE = 0,
    ADAM7 = 1,
}
pub const IDAT_TYPE: [u8; 4] = [0x49, 0x44, 0x41, 0x54];
