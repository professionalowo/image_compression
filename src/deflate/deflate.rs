#[derive(Debug, Clone, PartialEq, Eq)]
struct Compression(u8);

impl Compression {
    pub const fn new(level: u8) -> Self {
        Self(level)
    }
    pub const fn none() -> Self {
        Self::new(0)
    }

    /// Optimize for the best speed of encoding.
    pub const fn fast() -> Compression {
        Self::new(1)
    }

    /// Optimize for the size of data being encoded.
    pub const fn best() -> Compression {
        Self::new(9)
    }
}

impl Default for Compression {
    fn default() -> Self {
        Self::new(6)
    }
}
