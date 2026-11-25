use std::io::{self, Read, Write};

use super::ServiceWireError;

const VERSION: u8 = 1;
const REQUEST_HEADER: usize = 11;

use super::ServiceCallFrame;

/// Length-prefixed framing for sequential service calls over a byte stream.
#[derive(Debug, Clone, Copy)]
pub struct ServiceFrameCodec {
    max_frame_bytes: u32,
}

impl ServiceFrameCodec {
    /// Sets the maximum body size, excluding the four-byte length prefix.
    #[must_use]
    pub const fn new(max_frame_bytes: u32) -> Self {
        Self { max_frame_bytes }
    }

    /// Reads one bounded frame body. Returns `None` on clean EOF.
    ///
    /// If this method returns an error, close the stream rather than treating
    /// any remaining bytes as another frame.
    pub fn read_frame(&self, mut input: impl Read) -> io::Result<Option<Vec<u8>>> {
        let mut length = [0; 4];
        match input.read_exact(&mut length[..1]) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(error) => return Err(error),
        }
        input.read_exact(&mut length[1..])?;
        let length = u32::from_be_bytes(length);
        if length == 0 || length > self.max_frame_bytes {
            return Err(invalid("invalid service frame length"));
        }
        let length =
            usize::try_from(length).map_err(|_| invalid("invalid service frame length"))?;
        let mut frame = vec![0; length];
        input.read_exact(&mut frame)?;
        Ok(Some(frame))
    }

    /// Decodes exactly one complete length-prefixed packet without copying.
    pub fn decode_packet<'a>(&self, packet: &'a [u8]) -> io::Result<&'a [u8]> {
        let (prefix, body) = packet
            .split_first_chunk::<4>()
            .ok_or_else(|| invalid("truncated service frame length"))?;
        let length = u32::from_be_bytes(*prefix);
        if length == 0 || length > self.max_frame_bytes || usize::try_from(length) != Ok(body.len())
        {
            return Err(invalid("invalid service packet length"));
        }
        Ok(body)
    }

    /// Decodes a versioned request body without interpreting its JSON.
    pub fn decode_call(frame: &[u8]) -> io::Result<ServiceCallFrame<'_>> {
        let (&[version, handle @ .., high, low], content) = frame
            .split_first_chunk::<REQUEST_HEADER>()
            .ok_or_else(|| invalid("truncated service call header"))?;
        if version != VERSION {
            return Err(invalid("unsupported service frame version"));
        }
        let operation_size = usize::from(u16::from_be_bytes([high, low]));
        let (operation, payload) = content
            .split_at_checked(operation_size)
            .filter(|(operation, _)| !operation.is_empty())
            .ok_or_else(|| invalid("invalid service operation length"))?;
        let operation = std::str::from_utf8(operation)
            .map_err(|_| invalid("invalid UTF-8 service operation"))?;
        Ok(ServiceCallFrame {
            handle: u64::from_be_bytes(handle),
            operation,
            payload,
        })
    }

    /// Writes one call with a version, session handle, UTF-8 operation, and JSON payload.
    pub fn write_call(&self, output: impl Write, call: ServiceCallFrame<'_>) -> io::Result<()> {
        let size = u16::try_from(call.operation.len())
            .ok()
            .filter(|size| *size != 0)
            .ok_or_else(|| invalid("invalid service operation length"))?;
        self.write_parts(
            output,
            &[
                &[VERSION],
                &call.handle.to_be_bytes(),
                &size.to_be_bytes(),
                call.operation.as_bytes(),
                call.payload,
            ],
        )
    }

    /// Decodes a reply body into either JSON bytes or a stable service error.
    pub fn decode_reply(frame: &[u8]) -> io::Result<Result<&[u8], ServiceWireError>> {
        match frame.split_first_chunk::<2>() {
            Some(([VERSION, 0], payload)) => Ok(Ok(payload)),
            Some(([VERSION, 1], payload)) => serde_json::from_slice(payload)
                .map(Err)
                .map_err(|_| invalid("invalid service error code")),
            _ => Err(invalid("invalid service reply header")),
        }
    }

    /// Writes one JSON success reply or redacted error.
    pub fn write_reply(
        &self,
        output: impl Write,
        reply: Result<&[u8], ServiceWireError>,
    ) -> io::Result<()> {
        match reply {
            Ok(payload) => self.write_parts(output, &[&[VERSION, 0], payload]),
            Err(error) => {
                let payload = serde_json::to_vec(&error)?;
                self.write_parts(output, &[&[VERSION, 1], &payload])
            }
        }
    }

    fn write_parts(&self, mut output: impl Write, parts: &[&[u8]]) -> io::Result<()> {
        let length = parts
            .iter()
            .fold(0usize, |total, part| total.saturating_add(part.len()));
        let length = u32::try_from(length)
            .ok()
            .filter(|length| *length <= self.max_frame_bytes)
            .ok_or_else(|| invalid("service frame exceeds byte budget"))?;
        output.write_all(&length.to_be_bytes())?;
        for part in parts {
            output.write_all(part)?;
        }
        Ok(())
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
