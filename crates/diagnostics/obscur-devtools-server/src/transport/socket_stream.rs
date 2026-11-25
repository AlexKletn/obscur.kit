#![doc = include_str!("../../docs/api/transport/socket_stream.doc/_module.md")]

mod kernel_stream;

use kernel_stream::KernelStream;

mod socket_read_state;

use socket_read_state::SocketReadState;

use std::io::{self, Read, Write};
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::time::Duration;

#[doc = include_str!("../../docs/api/transport/socket_stream.doc/MAX_FRAME_LEN.md")]
pub const MAX_FRAME_LEN: usize = 1 << 20;

#[doc = include_str!("../../docs/api/transport/socket_stream.doc/SocketStream.md")]
#[derive(Debug)]
pub struct SocketStream {
    inner: KernelStream,
    #[doc = include_str!("../../docs/api/transport/socket_stream.doc/reading.md")]
    reading: SocketReadState,
}

impl SocketStream {
    /// Wraps an accepted or connected loopback TCP stream.
    #[must_use]
    pub fn from_tcp(stream: TcpStream) -> Self {
        Self {
            inner: KernelStream::Tcp(stream),
            reading: SocketReadState::default(),
        }
    }

    /// Wraps an accepted or connected Unix domain stream.
    #[cfg(unix)]
    #[must_use]
    pub fn from_unix(stream: UnixStream) -> Self {
        Self {
            inner: KernelStream::Unix(stream),
            reading: SocketReadState::default(),
        }
    }

    #[doc = include_str!("../../docs/api/transport/socket_stream.doc/set_read_timeout.md")]
    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        match &self.inner {
            KernelStream::Tcp(stream) => stream.set_read_timeout(timeout),
            #[cfg(unix)]
            KernelStream::Unix(stream) => stream.set_read_timeout(timeout),
        }
    }

    #[doc = include_str!("../../docs/api/transport/socket_stream.doc/try_clone.md")]
    pub fn try_clone(&self) -> io::Result<Self> {
        let inner = match &self.inner {
            KernelStream::Tcp(stream) => KernelStream::Tcp(stream.try_clone()?),
            #[cfg(unix)]
            KernelStream::Unix(stream) => KernelStream::Unix(stream.try_clone()?),
        };
        // The clone starts its own read state: partial progress belongs to the
        // handle that made it, and two handles reading one socket would interleave
        // bytes anyway.
        Ok(Self {
            inner,
            reading: SocketReadState::default(),
        })
    }

    #[doc = include_str!("../../docs/api/transport/socket_stream.doc/write_frame.md")]
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

    #[doc = include_str!("../../docs/api/transport/socket_stream.doc/read_frame.md")]
    pub fn read_frame(&mut self) -> io::Result<Option<Vec<u8>>> {
        loop {
            match &mut self.reading {
                SocketReadState::Header { buf, filled } => {
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
                    self.reading = SocketReadState::Body {
                        buf: vec![0u8; len],
                        filled: 0,
                    };
                }
                SocketReadState::Body { buf, filled } => {
                    // Never a clean EOF: the header promised these bytes.
                    if !fill(&mut self.inner, buf, filled, false)? {
                        return Ok(None);
                    }
                    let body = std::mem::take(buf);
                    self.reading = SocketReadState::default();
                    return Ok(Some(body));
                }
            }
        }
    }
}

#[doc = include_str!("../../docs/api/transport/socket_stream.doc/fill.md")]
fn fill(
    inner: &mut KernelStream,
    buf: &mut [u8],
    filled: &mut usize,
    at_start: bool,
) -> io::Result<bool> {
    while *filled < buf.len() {
        let read = match inner {
            KernelStream::Tcp(stream) => stream.read(&mut buf[*filled..]),
            #[cfg(unix)]
            KernelStream::Unix(stream) => stream.read(&mut buf[*filled..]),
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
            KernelStream::Tcp(stream) => stream.read(buf),
            #[cfg(unix)]
            KernelStream::Unix(stream) => stream.read(buf),
        }
    }
}

impl Write for SocketStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.inner {
            KernelStream::Tcp(stream) => stream.write(buf),
            #[cfg(unix)]
            KernelStream::Unix(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.inner {
            KernelStream::Tcp(stream) => stream.flush(),
            #[cfg(unix)]
            KernelStream::Unix(stream) => stream.flush(),
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
