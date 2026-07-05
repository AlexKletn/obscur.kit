//! `SocketStream`: a length-prefixed framed byte channel over a real local
//! kernel socket (spec §7.6 local socket transport).
//!
//! This is the low-level connection shared by the server-side
//! [`SocketTransport`](crate::transport::SocketTransport) and the client-side
//! [`SocketClient`](crate::transport::SocketClient). It wraps either a Unix
//! domain stream (preferred on Unix) or a `127.0.0.1` TCP stream (the portable
//! fallback) and frames the raw message bodies produced by
//! [`WireCodec`](crate::transport::WireCodec) as a `u32` little-endian length
//! prefix followed by that many payload bytes.
//!
//! Frames are bounded by [`MAX_FRAME_LEN`]: a declared length above the cap is
//! refused before any allocation, so a peer cannot drive unbounded memory use
//! (spec §7.7 limits). The socket is only ever bound to a loopback address or a
//! filesystem path — never a public interface.

use std::io::{self, Read, Write};
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

/// The largest single frame payload the transport will read, in bytes.
///
/// Sized to comfortably hold any well-formed DevTools message while still
/// rejecting a hostile or corrupt length prefix before allocating (spec §7.7).
pub const MAX_FRAME_LEN: usize = 1 << 20;

/// One end of a framed, local-only DevTools socket connection.
///
/// Reads and writes whole [`WireCodec`](crate::transport::WireCodec) message
/// bodies, delimiting each with a `u32` little-endian length prefix.
#[derive(Debug)]
pub struct SocketStream {
    inner: Inner,
}

/// The concrete kernel stream behind a [`SocketStream`].
#[derive(Debug)]
enum Inner {
    /// A loopback TCP stream (the portable fallback).
    Tcp(TcpStream),
    /// A Unix domain stream (preferred on Unix).
    #[cfg(unix)]
    Unix(UnixStream),
}

impl SocketStream {
    /// Wraps an accepted or connected loopback TCP stream.
    #[must_use]
    pub fn from_tcp(stream: TcpStream) -> Self {
        Self { inner: Inner::Tcp(stream) }
    }

    /// Wraps an accepted or connected Unix domain stream.
    #[cfg(unix)]
    #[must_use]
    pub fn from_unix(stream: UnixStream) -> Self {
        Self { inner: Inner::Unix(stream) }
    }

    /// Writes one framed message body: a `u32` little-endian length prefix
    /// followed by `body`.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if `body` exceeds [`MAX_FRAME_LEN`] or the
    /// underlying socket write fails.
    pub fn write_frame(&mut self, body: &[u8]) -> io::Result<()> {
        if body.len() > MAX_FRAME_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "frame exceeds MAX_FRAME_LEN",
            ));
        }
        let len = u32::try_from(body.len()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "frame length overflows u32")
        })?;
        self.write_all(&len.to_le_bytes())?;
        self.write_all(body)?;
        self.flush()
    }

    /// Reads one framed message body, or `None` on a clean end-of-stream.
    ///
    /// Returns `Ok(None)` only when the peer closed the connection cleanly at a
    /// frame boundary (no bytes of a length prefix were available).
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] on a truncated frame, a length above
    /// [`MAX_FRAME_LEN`], or any underlying socket read failure.
    pub fn read_frame(&mut self) -> io::Result<Option<Vec<u8>>> {
        let mut len_bytes = [0u8; 4];
        if !self.read_full_or_eof(&mut len_bytes)? {
            return Ok(None);
        }
        let len = usize::try_from(u32::from_le_bytes(len_bytes)).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "frame length overflows usize")
        })?;
        if len > MAX_FRAME_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "declared frame length exceeds MAX_FRAME_LEN",
            ));
        }
        let mut body = vec![0u8; len];
        self.read_exact(&mut body)?;
        Ok(Some(body))
    }

    /// Fills `buf` completely, returning `Ok(false)` if the stream ended cleanly
    /// before the first byte, and an error if it ended mid-buffer.
    fn read_full_or_eof(&mut self, buf: &mut [u8]) -> io::Result<bool> {
        let mut filled = 0;
        while filled < buf.len() {
            match self.read(&mut buf[filled..]) {
                Ok(0) if filled == 0 => return Ok(false),
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "stream ended mid-frame-header",
                    ));
                }
                Ok(n) => filled += n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(true)
    }
}

impl Read for SocketStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.inner {
            Inner::Tcp(stream) => stream.read(buf),
            #[cfg(unix)]
            Inner::Unix(stream) => stream.read(buf),
        }
    }
}

impl Write for SocketStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.inner {
            Inner::Tcp(stream) => stream.write(buf),
            #[cfg(unix)]
            Inner::Unix(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.inner {
            Inner::Tcp(stream) => stream.flush(),
            #[cfg(unix)]
            Inner::Unix(stream) => stream.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};

    #[test]
    fn frames_round_trip_over_loopback_tcp() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback");
        let addr = listener.local_addr().expect("addr");
        let handle = std::thread::spawn(move || {
            let (server, _) = listener.accept().expect("accept");
            let mut server = SocketStream::from_tcp(server);
            let frame = server.read_frame().expect("read").expect("frame");
            assert_eq!(frame, b"hello");
            server.write_frame(b"world").expect("write");
        });

        let mut client = SocketStream::from_tcp(TcpStream::connect(addr).expect("connect"));
        client.write_frame(b"hello").expect("write");
        let reply = client.read_frame().expect("read").expect("frame");
        assert_eq!(reply, b"world");
        handle.join().expect("join");
    }

    #[test]
    fn clean_eof_yields_none() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback");
        let addr = listener.local_addr().expect("addr");
        let handle = std::thread::spawn(move || {
            let (server, _) = listener.accept().expect("accept");
            let mut server = SocketStream::from_tcp(server);
            assert_eq!(server.read_frame().expect("read"), None);
        });
        // Connect then immediately drop to close cleanly at a frame boundary.
        let client = TcpStream::connect(addr).expect("connect");
        drop(client);
        handle.join().expect("join");
    }

    #[test]
    fn oversized_frame_is_refused_before_alloc() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback");
        let addr = listener.local_addr().expect("addr");
        let handle = std::thread::spawn(move || {
            let (server, _) = listener.accept().expect("accept");
            let mut server = SocketStream::from_tcp(server);
            let err = server.read_frame().expect_err("oversized");
            assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        });
        let mut raw = TcpStream::connect(addr).expect("connect");
        let bogus = u32::try_from(MAX_FRAME_LEN + 1).unwrap();
        raw.write_all(&bogus.to_le_bytes()).expect("write len");
        raw.flush().expect("flush");
        handle.join().expect("join");
    }
}
