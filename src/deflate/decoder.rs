use std::io::Error;

#[derive(Debug)]
pub struct Decoder {
    data: Box<[u8]>,
}

impl Decoder {
    pub fn new<I>(data_source: I) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        let data: Box<[u8]> = data_source.into_iter().collect();
        Self { data }
    }

    pub fn decode(&self) -> Result<Box<[u8]>, Error> {
        Ok(Box::new([0]))
    }
}
