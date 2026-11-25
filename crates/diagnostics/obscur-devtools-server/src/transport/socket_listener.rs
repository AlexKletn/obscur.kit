#![doc = include_str!("../../docs/api/transport/socket_listener.doc/_module.md")]

mod kernel_listener;

use kernel_listener::KernelListener;

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
    inner: KernelListener,
}

impl SocketListener {
    #[doc = include_str!("../../docs/api/transport/socket_listener.doc/bind_loopback.md")]
    pub fn bind_loopback() -> io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        Ok(Self {
            inner: KernelListener::Tcp(listener),
        })
    }

    #[doc = include_str!("../../docs/api/transport/socket_listener.doc/bind_unix.md")]
    #[cfg(unix)]
    pub fn bind_unix(path: impl AsRef<Path>) -> io::Result<Self> {
        let listener = UnixListener::bind(path)?;
        Ok(Self {
            inner: KernelListener::Unix(listener),
        })
    }

    #[doc = include_str!("../../docs/api/transport/socket_listener.doc/local_addr.md")]
    pub fn local_addr(&self) -> io::Result<Option<SocketAddr>> {
        match &self.inner {
            KernelListener::Tcp(listener) => listener.local_addr().map(Some),
            #[cfg(unix)]
            KernelListener::Unix(_) => Ok(None),
        }
    }

    #[doc = include_str!("../../docs/api/transport/socket_listener.doc/accept.md")]
    pub fn accept(&self) -> io::Result<SocketStream> {
        match &self.inner {
            KernelListener::Tcp(listener) => {
                let (stream, _peer) = listener.accept()?;
                Ok(SocketStream::from_tcp(stream))
            }
            #[cfg(unix)]
            KernelListener::Unix(listener) => {
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
