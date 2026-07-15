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
use std::time::Duration;

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
    /// How far the in-progress [`read_frame`](Self::read_frame) has got.
    ///
    /// A framed reader and a read timeout do not mix unless the reader is
    /// **resumable**: a timeout can land after some of a frame has arrived, and
    /// discarding those bytes would make the next call read the frame's own tail as
    /// a length prefix — desyncing the stream permanently. Progress therefore lives
    /// here, across calls, so a timed-out read resumes exactly where it stopped.
    reading: ReadState,
}

/// How far the current frame has been read.
#[derive(Debug)]
enum ReadState {
    /// Reading the `u32` length prefix; `filled` bytes of it have arrived.
    Header { buf: [u8; 4], filled: usize },
    /// The length is known; `filled` bytes of the body have arrived.
    Body { buf: Vec<u8>, filled: usize },
}

impl Default for ReadState {
    fn default() -> Self {
        Self::Header {
            buf: [0; 4],
            filled: 0,
        }
    }
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
        Self {
            inner: Inner::Tcp(stream),
            reading: ReadState::default(),
        }
    }

    /// Wraps an accepted or connected Unix domain stream.
    #[cfg(unix)]
    #[must_use]
    pub fn from_unix(stream: UnixStream) -> Self {
        Self {
            inner: Inner::Unix(stream),
            reading: ReadState::default(),
        }
    }

    /// Sets how long a [`read_frame`](Self::read_frame) waits before giving up,
    /// or clears the timeout with [`None`].
    ///
    /// A serving loop needs this to stay responsive to work that is not a client
    /// message: without a timeout it blocks in `read` forever, so an event a host
    /// wants to *push* could never be written until the client happened to send
    /// something first — which is not a push at all.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the underlying socket rejects the timeout.
    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        match &self.inner {
            Inner::Tcp(stream) => stream.set_read_timeout(timeout),
            #[cfg(unix)]
            Inner::Unix(stream) => stream.set_read_timeout(timeout),
        }
    }

    /// Clones the handle so one side can read while another writes.
    ///
    /// Both handles refer to the same kernel socket, which is full-duplex: a push
    /// written on the clone lands even while the original is parked in `read`.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the underlying socket cannot be duplicated.
    pub fn try_clone(&self) -> io::Result<Self> {
        let inner = match &self.inner {
            Inner::Tcp(stream) => Inner::Tcp(stream.try_clone()?),
            #[cfg(unix)]
            Inner::Unix(stream) => Inner::Unix(stream.try_clone()?),
        };
        // The clone starts its own read state: partial progress belongs to the
        // handle that made it, and two handles reading one socket would interleave
        // bytes anyway.
        Ok(Self {
            inner,
            reading: ReadState::default(),
        })
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
        loop {
            match &mut self.reading {
                ReadState::Header { buf, filled } => {
                    // A clean EOF is only clean *between* frames.
                    let at_start = *filled == 0;
                    if !fill(&mut self.inner, buf, filled, at_start)? {
                        return Ok(None);
                    }
                    let len = usize::try_from(u32::from_le_bytes(*buf)).map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidData, "frame length overflows usize")
                    })?;
                    if len > MAX_FRAME_LEN {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "declared frame length exceeds MAX_FRAME_LEN",
                        ));
                    }
                    self.reading = ReadState::Body {
                        buf: vec![0u8; len],
                        filled: 0,
                    };
                }
                ReadState::Body { buf, filled } => {
                    // Never a clean EOF: the header promised these bytes.
                    if !fill(&mut self.inner, buf, filled, false)? {
                        return Ok(None);
                    }
                    let body = std::mem::take(buf);
                    self.reading = ReadState::default();
                    return Ok(Some(body));
                }
            }
        }
    }
}

/// Fills `buf` from `filled` onward, advancing `filled` by whatever arrives.
///
/// Returns `Ok(true)` once `buf` is full, `Ok(false)` on a clean end of stream —
/// which is only clean when `at_start`, i.e. nothing of this frame had arrived
/// yet; an EOF mid-frame is a truncated message, not a polite disconnect.
///
/// **`filled` is the caller's state, not a local.** On a would-block/timeout the
/// error propagates with `filled` already advanced past the bytes that did land,
/// so the next call resumes rather than re-reading them as a fresh frame header.
fn fill(inner: &mut Inner, buf: &mut [u8], filled: &mut usize, at_start: bool) -> io::Result<bool> {
    while *filled < buf.len() {
        let read = match inner {
            Inner::Tcp(stream) => stream.read(&mut buf[*filled..]),
            #[cfg(unix)]
            Inner::Unix(stream) => stream.read(&mut buf[*filled..]),
        };
        match read {
            Ok(0) if at_start && *filled == 0 => return Ok(false),
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "stream ended mid-frame",
                ));
            }
            Ok(n) => *filled += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(true)
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
