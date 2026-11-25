use std::io::{self, Read};

pub(super) struct ByteLimit<R> {
    input: R,
    remaining: u64,
}

impl<R> ByteLimit<R> {
    pub(super) const fn new(input: R, remaining: u64) -> Self {
        Self { input, remaining }
    }
}

impl<R: Read> Read for ByteLimit<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        if self.remaining == 0 {
            return match self.input.read(&mut [0])? {
                0 => Ok(0),
                _ => Err(io::Error::other("package stream byte limit exceeded")),
            };
        }
        let count = buffer
            .len()
            .min(usize::try_from(self.remaining).unwrap_or(usize::MAX));
        let n = self.input.read(&mut buffer[..count])?;
        self.remaining -= u64::try_from(n).expect("read is bounded by the remaining byte budget");
        Ok(n)
    }
}
