use obscur_devtools_protocol::message::{CommandMessage, HandshakeRequest};

#[doc = include_str!("../../docs/api/transport/local_transport.doc/ServerBound.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerBound {
    /// A connection handshake request.
    Handshake(HandshakeRequest),
    /// A capability-scoped command.
    Command(CommandMessage),
}
