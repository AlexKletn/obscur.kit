#![doc = include_str!("../../docs/api/transport/socket_client.doc/_module.md")]

use std::io;
use std::net::{SocketAddr, TcpStream};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(unix)]
use std::path::Path;

use obscur_devtools_protocol::message::{CommandMessage, HandshakeRequest, HandshakeResponse};

use crate::transport::{ClientBound, ServerBound, SocketStream, WireCodec};

/// The client end of a framed DevTools socket connection.
#[derive(Debug)]
pub struct SocketClient {
    stream: SocketStream,
}

impl SocketClient {
    #[doc = include_str!("../../docs/api/transport/socket_client.doc/connect_loopback.md")]
    pub fn connect_loopback(addr: SocketAddr) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(Self {
            stream: SocketStream::from_tcp(stream),
        })
    }

    #[doc = include_str!("../../docs/api/transport/socket_client.doc/connect_unix.md")]
    #[cfg(unix)]
    pub fn connect_unix(path: impl AsRef<Path>) -> io::Result<Self> {
        let stream = UnixStream::connect(path)?;
        Ok(Self {
            stream: SocketStream::from_unix(stream),
        })
    }

    #[doc = include_str!("../../docs/api/transport/socket_client.doc/handshake.md")]
    pub fn handshake(&mut self, request: &HandshakeRequest) -> io::Result<HandshakeResponse> {
        match self.exchange(&ServerBound::Handshake(request.clone()))? {
            ClientBound::Handshake(response) => Ok(response),
            other => Err(unexpected(&format!(
                "expected handshake response, got {other:?}"
            ))),
        }
    }

    #[doc = include_str!("../../docs/api/transport/socket_client.doc/command.md")]
    pub fn command(&mut self, command: &CommandMessage) -> io::Result<ClientBound> {
        self.exchange(&ServerBound::Command(command.clone()))
    }

    #[doc = include_str!("../../docs/api/transport/socket_client.doc/receive.md")]
    pub fn receive(&mut self) -> io::Result<Option<ClientBound>> {
        let Some(frame) = self.stream.read_frame()? else {
            return Ok(None);
        };
        WireCodec::decode_client_bound(&frame)
            .map(Some)
            .map_err(|error| unexpected(&error.to_string()))
    }

    #[doc = include_str!("../../docs/api/transport/socket_client.doc/set_read_timeout.md")]
    pub fn set_read_timeout(&mut self, timeout: Option<std::time::Duration>) -> io::Result<()> {
        self.stream.set_read_timeout(timeout)
    }

    /// Writes one server-bound message and reads exactly one reply.
    fn exchange(&mut self, message: &ServerBound) -> io::Result<ClientBound> {
        self.stream
            .write_frame(&WireCodec::encode_server_bound(message))?;
        let frame = self
            .stream
            .read_frame()?
            .ok_or_else(|| unexpected("server closed the connection before replying"))?;
        WireCodec::decode_client_bound(&frame).map_err(|error| unexpected(&error.to_string()))
    }
}

/// Builds an `InvalidData` I/O error describing a protocol violation.
fn unexpected(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::Authenticator;
    use crate::bridge::{NullInspectSource, RuntimeBridge};
    use crate::endpoint::DevToolsEndpoint;
    use crate::transport::{SocketListener, SocketTransport};
    use obscur_devtools_protocol::message::{ProtocolVersion, ScopeSet, VersionRange};

    #[test]
    fn client_handshake_reaches_a_disabled_endpoint_and_is_rejected() {
        // §7.7 disabled-by-default: the endpoint is never enabled, so a real
        // client's handshake must come back rejected over the socket.
        let listener = SocketListener::bind_loopback().expect("bind");
        let addr = listener.local_addr().expect("addr").expect("tcp addr");
        let handle = std::thread::spawn(move || {
            let mut auth = Authenticator::new();
            auth.register_token("dev", ScopeSet::all());
            let mut endpoint = DevToolsEndpoint::new(auth, RuntimeBridge::new(NullInspectSource));
            // NOTE: deliberately not enabled.
            let stream = listener.accept().expect("accept");
            SocketTransport::new(stream)
                .serve(&mut endpoint)
                .expect("serve");
        });

        let mut client = SocketClient::connect_loopback(addr).expect("connect");
        let request = HandshakeRequest::new(VersionRange::exact(ProtocolVersion::CURRENT), "dev");
        let response = client.handshake(&request).expect("handshake io");
        assert!(!response.is_accepted());
        drop(client);
        handle.join().expect("join");
    }
}
