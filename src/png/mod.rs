use crate::Compressable;

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
    fn compress(&self) -> Self {
        let data = self.data.clone();

        //strip auxilliary chunks
        let only_data = data
            .iter()
            .filter_map(|chunk| match chunk.is_ancillary() {
                false => Some(chunk.clone()),
                _ => None,
            })
            .collect();

        let mut out = self.clone();

        out.data = only_data;

        out
    }
}

impl<I> From<I> for PngImage
where
    I: IntoIterator<Item = u8>,
{
    fn from(data: I) -> Self {
        Self::try_create(data).unwrap()
    }
}

impl Into<Vec<u8>> for PngImage {
    fn into(self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        vec.extend(self.file_signature);
        vec.extend(
            self.data
                .iter()
                .flat_map(move |c| -> Vec<u8> { Chunk::into(c.clone()) })
                .collect::<Vec<u8>>(),
        );

        vec
    }
}
#[derive(Debug, Clone)]
pub struct IhdrHeader {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub compression_method: CompressionMethod,
    pub filter_method: FilterMethod,
    pub interlace_method: InterlaceMethod,
}

impl TryFrom<Chunk> for IhdrHeader {
    type Error = PngError;
    fn try_from(value: Chunk) -> PngResult<Self> {
        let bytes = value.chunk_data;

        let width_bytes: [u8; 4] = match bytes[0..4].try_into() {
            Ok(b) => b,
            Err(_) => return Err(PngError::new("not enough data")),
        };

        let width = u32::from_be_bytes(width_bytes);

        let height_bytes: [u8; 4] = match bytes[4..8].try_into() {
            Ok(b) => b,
            Err(_) => return Err(PngError::new("not enough data")),
        };

        let height = u32::from_be_bytes(height_bytes);

        let bit_depth = bytes[8];
        let color_type: ColorType = bytes[9].try_into()?;
        let compression_method: CompressionMethod = match bytes[10] {
            0 => CompressionMethod::DEFLATE,
            _ => return Err(PngError::new("unknown compression method")),
        };
        let filter_method: FilterMethod = bytes[11].try_into()?;
        let interlace_method: InterlaceMethod = match bytes[12] {
            0 => InterlaceMethod::NONE,
            1 => InterlaceMethod::ADAM7,
            _ => return Err(PngError::new("unknown interlace method")),
        };

        let header = IhdrHeader {
            width,
            height,
            bit_depth,
            color_type,
            compression_method,
            filter_method,
            interlace_method,
        };
        Ok(header)
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub length: u32,
    chunk_type: [u8; 4],
    chunk_data: Box<[u8]>,
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
    fn try_create<I>(length: u32, data: I) -> PngResult<Self>
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

        let chunk = Chunk {
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
