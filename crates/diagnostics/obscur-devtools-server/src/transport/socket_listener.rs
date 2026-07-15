//! `SocketListener`: a local-only listening socket for DevTools connections
//! (spec §7.6 local socket transport, §7.7 local-only).
//!
//! The listener binds either a Unix domain socket (preferred on Unix) or a
//! `127.0.0.1` loopback TCP socket (the portable fallback) and accepts client
//! connections, handing each back as a framed [`SocketStream`]. It never binds a
//! routable interface: TCP is pinned to loopback and the Unix variant is a
//! filesystem path, keeping the endpoint local-only per §7.7. Enabling and
//! authenticating a session remains entirely the endpoint's responsibility — the
//! listener only moves bytes.

use std::io;
use std::net::{SocketAddr, TcpListener};
#[cfg(unix)]
use std::os::unix::net::UnixListener;
#[cfg(unix)]
use std::path::Path;

use crate::transport::SocketStream;

/// A local-only listening socket that accepts DevTools client connections.
#[derive(Debug)]
pub struct SocketListener {
    inner: Inner,
}

/// The concrete kernel listener behind a [`SocketListener`].
#[derive(Debug)]
enum Inner {
    /// A loopback-bound TCP listener (the portable fallback).
    Tcp(TcpListener),
    /// A Unix domain listener bound to a filesystem path.
    #[cfg(unix)]
    Unix(UnixListener),
}

impl SocketListener {
    /// Binds a TCP listener to an ephemeral `127.0.0.1` port.
    ///
    /// Loopback-only: the socket is never reachable off the local host. Use
    /// [`local_addr`](Self::local_addr) to discover the chosen port for a client
    /// to connect to.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the loopback bind fails.
    pub fn bind_loopback() -> io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        Ok(Self {
            inner: Inner::Tcp(listener),
        })
    }

    /// Binds a Unix domain listener to `path`.
    ///
    /// A filesystem-scoped socket is local by construction. The caller owns the
    /// path's lifetime (a temp directory keeps tests hermetic).
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the path is already bound or the bind fails.
    #[cfg(unix)]
    pub fn bind_unix(path: impl AsRef<Path>) -> io::Result<Self> {
        let listener = UnixListener::bind(path)?;
        Ok(Self {
            inner: Inner::Unix(listener),
        })
    }

    /// Returns the bound loopback address, if this is a TCP listener.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if the address cannot be queried.
    pub fn local_addr(&self) -> io::Result<Option<SocketAddr>> {
        match &self.inner {
            Inner::Tcp(listener) => listener.local_addr().map(Some),
            #[cfg(unix)]
            Inner::Unix(_) => Ok(None),
        }
    }

    /// Blocks until one client connects, returning its framed stream.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if accepting the connection fails.
    pub fn accept(&self) -> io::Result<SocketStream> {
        match &self.inner {
            Inner::Tcp(listener) => {
                let (stream, _peer) = listener.accept()?;
                Ok(SocketStream::from_tcp(stream))
            }
            #[cfg(unix)]
            Inner::Unix(listener) => {
                let (stream, _peer) = listener.accept()?;
                Ok(SocketStream::from_unix(stream))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_listener_reports_a_local_port() {
        let listener = SocketListener::bind_loopback().expect("bind");
        let addr = listener.local_addr().expect("addr").expect("tcp addr");
        assert!(addr.ip().is_loopback());
        assert_ne!(addr.port(), 0);
    }
}
