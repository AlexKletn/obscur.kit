//! In-process loopback transport and its wire envelopes (spec §7.6/§7.7).

use std::collections::VecDeque;

use obscur_devtools_protocol::error::ProtocolError;
use obscur_devtools_protocol::message::{
    CommandId, CommandMessage, HandshakeRequest, HandshakeResponse, MESSAGE_LIMIT, ProtocolEvent,
    SnapshotMessage, Subscription,
};

/// A message travelling from a DevTools client to the server.
///
/// The transport is direction-typed: only handshakes and commands flow toward
/// the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerBound {
    /// A connection handshake request.
    Handshake(HandshakeRequest),
    /// A capability-scoped command.
    Command(CommandMessage),
}

/// A message travelling from the server to a DevTools client.
///
/// Carries handshake results, snapshot responses, pushed events, subscription
/// records, command acknowledgements, and protocol errors.
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

/// An in-process, loopback message channel between a DevTools client and server.
///
/// This models the spec's *local socket* transport (spec §7.6) without any real
/// IO: two bounded queues carry [`ServerBound`] messages one way and
/// [`ClientBound`] messages the other. A real socket transport would expose the
/// same shape (`send_to_server` / `poll_server` / `push_to_client` /
/// `poll_client`) over a kernel socket. Both queues enforce
/// [`MESSAGE_LIMIT`](obscur_devtools_protocol::message::MESSAGE_LIMIT):
/// enqueueing onto a full queue fails with
/// [`ProtocolError::LimitExceeded`] rather than growing unboundedly (spec §7.7).
#[derive(Debug, Clone, Default)]
pub struct LocalTransport {
    to_server: VecDeque<ServerBound>,
    to_client: VecDeque<ClientBound>,
}

impl LocalTransport {
    /// Creates an empty transport with both queues drained.
    #[must_use]
    pub fn new() -> Self {
        Self {
            to_server: VecDeque::new(),
            to_client: VecDeque::new(),
        }
    }

    /// Enqueues a client→server message.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::LimitExceeded`] when the server-bound queue
    /// already holds [`MESSAGE_LIMIT`] messages.
    pub fn send_to_server(&mut self, message: ServerBound) -> Result<(), ProtocolError> {
        if self.to_server.len() >= MESSAGE_LIMIT {
            return Err(ProtocolError::LimitExceeded {
                limit: MESSAGE_LIMIT,
            });
        }
        self.to_server.push_back(message);
        Ok(())
    }

    /// Removes and returns the next client→server message, if any.
    #[must_use]
    pub fn poll_server(&mut self) -> Option<ServerBound> {
        self.to_server.pop_front()
    }

    /// Enqueues a server→client message.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::LimitExceeded`] when the client-bound queue
    /// already holds [`MESSAGE_LIMIT`] messages.
    pub fn push_to_client(&mut self, message: ClientBound) -> Result<(), ProtocolError> {
        if self.to_client.len() >= MESSAGE_LIMIT {
            return Err(ProtocolError::LimitExceeded {
                limit: MESSAGE_LIMIT,
            });
        }
        self.to_client.push_back(message);
        Ok(())
    }

    /// Removes and returns the next server→client message, if any.
    #[must_use]
    pub fn poll_client(&mut self) -> Option<ClientBound> {
        self.to_client.pop_front()
    }

    /// Returns the number of queued client→server messages.
    #[must_use]
    pub fn pending_to_server(&self) -> usize {
        self.to_server.len()
    }

    /// Returns the number of queued server→client messages.
    #[must_use]
    pub fn pending_to_client(&self) -> usize {
        self.to_client.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_devtools_protocol::message::{
        CommandKind, Inspectable, ProtocolVersion, SnapshotPayload, VersionRange,
    };

    #[test]
    fn round_trips_a_command_and_a_response() {
        let mut transport = LocalTransport::new();
        let command =
            CommandMessage::new(CommandId::new(1), CommandKind::Inspect(Inspectable::Tree));
        transport
            .send_to_server(ServerBound::Command(command.clone()))
            .expect("under limit");
        assert_eq!(transport.pending_to_server(), 1);
        assert_eq!(transport.poll_server(), Some(ServerBound::Command(command)));
        assert_eq!(transport.poll_server(), None);

        let snapshot = SnapshotMessage::new(
            CommandId::new(1),
            Inspectable::Tree,
            SnapshotPayload::summary("tree"),
        );
        transport
            .push_to_client(ClientBound::Snapshot(snapshot.clone()))
            .expect("under limit");
        assert_eq!(
            transport.poll_client(),
            Some(ClientBound::Snapshot(snapshot))
        );
        assert_eq!(transport.poll_client(), None);
    }

    #[test]
    fn handshake_is_server_bound() {
        let mut transport = LocalTransport::new();
        let request = HandshakeRequest::new(VersionRange::exact(ProtocolVersion::CURRENT), "token");
        transport
            .send_to_server(ServerBound::Handshake(request.clone()))
            .expect("under limit");
        assert_eq!(
            transport.poll_server(),
            Some(ServerBound::Handshake(request))
        );
    }

    #[test]
    fn server_queue_enforces_message_limit() {
        let mut transport = LocalTransport::new();
        let command =
            CommandMessage::new(CommandId::new(0), CommandKind::Inspect(Inspectable::Scene));
        for _ in 0..MESSAGE_LIMIT {
            transport
                .send_to_server(ServerBound::Command(command.clone()))
                .expect("within limit");
        }
        assert_eq!(transport.pending_to_server(), MESSAGE_LIMIT);
        assert_eq!(
            transport.send_to_server(ServerBound::Command(command)),
            Err(ProtocolError::LimitExceeded {
                limit: MESSAGE_LIMIT
            })
        );
    }

    #[test]
    fn client_queue_enforces_message_limit() {
        let mut transport = LocalTransport::new();
        for _ in 0..MESSAGE_LIMIT {
            transport
                .push_to_client(ClientBound::Acknowledged(CommandId::new(0)))
                .expect("within limit");
        }
        assert_eq!(
            transport.push_to_client(ClientBound::Acknowledged(CommandId::new(0))),
            Err(ProtocolError::LimitExceeded {
                limit: MESSAGE_LIMIT
            })
        );
    }
}
