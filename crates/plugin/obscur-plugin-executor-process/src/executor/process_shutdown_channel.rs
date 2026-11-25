//! One-shot shutdown control connection for a process plugin.

use std::{
    io::{self, Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};

use obscur_plugin_model::lifecycle::ShutdownConcern;

/// Exchanges a shutdown request and advisory concern over a dedicated TCP connection supplied by the host.
pub struct ProcessShutdownChannel {
    stream: TcpStream,
    max_message_bytes: u32,
}

impl ProcessShutdownChannel {
    /// Takes exclusive ownership of a connected stream. The host must authenticate it as belonging to the intended plugin. The cap applies to the UTF-8 message in the reply.
    #[must_use]
    pub fn new(stream: TcpStream, max_message_bytes: u32) -> Self {
        Self {
            stream,
            max_message_bytes,
        }
    }

    /// Sends a shutdown request and reads the concern under one deadline. The channel closes on return. Run on the process worker.
    /// The request is byte 1 followed by the remaining milliseconds as a little-endian u64. A reply byte of 0 means no concern. A reply byte of 1 is followed by requested milliseconds as a little-endian u64, a little-endian u32 byte length, and the UTF-8 message.
    pub fn request(mut self, deadline: Instant) -> io::Result<Option<ShutdownConcern>> {
        let budget = remaining(deadline)?;
        let budget_ms = u64::try_from(budget.as_millis()).unwrap_or(u64::MAX);
        self.stream.set_nonblocking(false)?;
        let mut request = [0; 9];
        request[0] = 1;
        request[1..].copy_from_slice(&budget_ms.to_le_bytes());
        self.write_request(&request, deadline)?;

        let mut tag = [0];
        self.read_exact(&mut tag, deadline)?;
        match tag[0] {
            0 => Ok(None),
            1 => {
                let mut time = [0; 8];
                let mut length = [0; 4];
                self.read_exact(&mut time, deadline)?;
                self.read_exact(&mut length, deadline)?;
                let time = u64::from_le_bytes(time);
                let length = u32::from_le_bytes(length);
                if length > self.max_message_bytes {
                    return Err(invalid("shutdown concern exceeds its byte limit"));
                }
                let mut message = vec![0; length as usize];
                self.read_exact(&mut message, deadline)?;
                let message = String::from_utf8(message)
                    .map_err(|_| invalid("shutdown concern is not UTF-8"))?;
                Ok(Some(ShutdownConcern {
                    message: message.into_boxed_str(),
                    requested_time: Duration::from_millis(time).min(remaining(deadline)?),
                }))
            }
            _ => Err(invalid("invalid process shutdown reply")),
        }
    }

    fn write_request(&mut self, mut bytes: &[u8], deadline: Instant) -> io::Result<()> {
        while !bytes.is_empty() {
            self.stream.set_write_timeout(Some(remaining(deadline)?))?;
            match self.stream.write(bytes) {
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(written) => bytes = &bytes[written..],
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(normalize_timeout(error)),
            }
        }
        Ok(())
    }

    fn read_exact(&mut self, mut bytes: &mut [u8], deadline: Instant) -> io::Result<()> {
        while !bytes.is_empty() {
            self.stream.set_read_timeout(Some(remaining(deadline)?))?;
            match self.stream.read(bytes) {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(read) => bytes = &mut bytes[read..],
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => return Err(normalize_timeout(error)),
            }
        }
        remaining(deadline).map(|_| ())
    }
}

fn remaining(deadline: Instant) -> io::Result<Duration> {
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(left)
}

fn normalize_timeout(error: io::Error) -> io::Error {
    if error.kind() == io::ErrorKind::WouldBlock {
        return io::ErrorKind::TimedOut.into();
    }
    error
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
