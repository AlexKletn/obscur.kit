use obscur_devtools_protocol::error::ProtocolError;
use obscur_devtools_protocol::message::{
    CommandId, HandshakeResponse, ProtocolEvent, SnapshotMessage, Subscription,
};

#[doc = include_str!("../../docs/api/transport/local_transport.doc/ClientBound.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientBound {
    /// The server's response to a handshake.
    Handshake(HandshakeResponse),
    /// A snapshot answering an `Inspect` command.
    Snapshot(SnapshotMessage),
    /// An event pushed over an active subscription.
    Event(ProtocolEvent),
    /// A subscription record answering a `Subscribe` command.
    Subscription(Subscription),
    /// An acknowledgement that a command was applied, keyed by its id.
    Acknowledged(CommandId),
    /// A protocol-level error answering a command.
    Error(ProtocolError),
}
