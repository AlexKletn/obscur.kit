use std::io::{self, Write};

pub(super) struct BoundedResponse {
    pub(super) bytes: Vec<u8>,
    pub(super) limit: usize,
    pub(super) exceeded: bool,
}

impl Write for BoundedResponse {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit - self.bytes.len() {
            self.exceeded = true;
            return Err(io::Error::other("service response byte budget exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
