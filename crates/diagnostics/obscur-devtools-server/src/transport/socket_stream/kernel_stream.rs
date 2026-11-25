use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;

/// The concrete kernel stream behind a [`SocketStream`](super::SocketStream).
#[derive(Debug)]
pub(super) enum KernelStream {
    /// A loopback TCP stream (the portable fallback).
    Tcp(TcpStream),
    /// A Unix domain stream (preferred on Unix).
    #[cfg(unix)]
    Unix(UnixStream),
}
