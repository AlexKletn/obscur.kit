use std::net::TcpListener;
#[cfg(unix)]
use std::os::unix::net::UnixListener;

/// The concrete kernel listener behind a [`SocketListener`](super::SocketListener).
#[derive(Debug)]
pub(super) enum KernelListener {
    /// A loopback-bound TCP listener (the portable fallback).
    Tcp(TcpListener),
    /// A Unix domain listener bound to a filesystem path.
    #[cfg(unix)]
    Unix(UnixListener),
}
