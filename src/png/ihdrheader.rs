use super::{
    Chunk, ColorType, CompressionMethod, FilterMethod, InterlaceMethod, PngError, PngResult,
};

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
