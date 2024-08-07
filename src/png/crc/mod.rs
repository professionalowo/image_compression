pub struct Crc32 {
    crc_table: [u32; 256],
    crc_table_computed: bool,
}

impl Crc32 {
    pub fn new() -> Self {
        let mut crc32 = Crc32 {
            crc_table: [0; 256],
            crc_table_computed: false,
        };
        crc32.make_crc_table();
        crc32
    }

    fn make_crc_table(&mut self) {
        let mut c: u32;
        for n in 0..256 {
            c = n as u32;
            for _ in 0..8 {
                if c & 1 != 0 {
                    c = 0xedb88320 ^ (c >> 1);
                } else {
                    c = c >> 1;
                }
            }
            self.crc_table[n as usize] = c;
        }
        self.crc_table_computed = true;
    }

    pub fn update_crc(&self, crc: u32, buf: &[u8]) -> u32 {
        let mut c = crc;
        for &byte in buf {
            c = self.crc_table[((c ^ byte as u32) & 0xff) as usize] ^ (c >> 8);
        }
        c
    }

    pub fn crc(&self, buf: &[u8]) -> u32 {
        self.update_crc(0xffffffff, buf) ^ 0xffffffff
    }
}
