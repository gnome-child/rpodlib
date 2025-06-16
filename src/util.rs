use std::io::{Result, Seek, SeekFrom, Write};

pub(crate) struct ByteCounter {
    count: usize,
}

impl ByteCounter {
    pub fn new() -> Self {
        ByteCounter { count: 0 }
    }

    pub fn bytes(&self) -> u32 {
        self.count as u32
    }
}

impl Write for ByteCounter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let n = buf.len();
        self.count += n;
        Ok(n)
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Seek for ByteCounter {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let new = match pos {
            SeekFrom::Start(off) => off as i64,
            SeekFrom::End(delta) => self.count as i64 + delta,
            SeekFrom::Current(delta) => self.count as i64 + delta,
        };
        if new < 0 {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid seek position",
            ))
        } else {
            self.count = new as usize;
            Ok(self.count as u64)
        }
    }
}
