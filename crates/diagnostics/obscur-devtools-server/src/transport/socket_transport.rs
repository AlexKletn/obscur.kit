//! `SocketTransport`: the server-side driver that runs a [`DevToolsEndpoint`]
//! over a real local socket (spec §7.6 local socket transport, §7.7 security).
//!
//! This is the real counterpart to the in-process
//! [`LocalTransport`](crate::transport::LocalTransport): instead of moving typed
//! values between two [`VecDeque`](std::collections::VecDeque)s, it reads framed
//! [`WireCodec`](crate::transport::WireCodec) messages off a
//! [`SocketStream`], feeds each into the *unchanged* endpoint (handshake →
//! authorize → dispatch → audit), and writes the endpoint's typed response back.
//! The transport never bypasses authentication, never grants a scope, and
//! enables nothing: a fresh endpoint is still disabled by default and rejects
//! every handshake until the host calls [`enable`](DevToolsEndpoint::enable)
//! (spec §7.7).
//!
//! It writes exactly one bit of session state, and only
//! [`serve_with_events`](SocketTransport::serve_with_events) does: that this
//! session's events are being serviced, which is a fact about the loop the
//! caller is standing in rather than a permission. It cannot loosen an
//! authorisation — a `Subscribe` is still checked against the granted
//! [`ScopeSet`](obscur_devtools_protocol::message::ScopeSet) first, and this bit
//! only decides whether a scope-authorised subscription is honoured or denied.
//!
//! A single [`serve`](SocketTransport::serve) call handles one connected client
//! session in its own thread; the endpoint's session state (negotiated version,
//! granted scopes, subscriptions, audit log) persists across the messages of
//! that session. At most [`MESSAGE_LIMIT`] messages are served per connection,
//! honouring the §7.7 message limit the in-process transport enforces on its
//! queue depth.

use std::io;

use obscur_devtools_protocol::message::MESSAGE_LIMIT;

use crate::bridge::InspectSource;
use crate::endpoint::{CommandOutcome, DevToolsEndpoint};
use crate::transport::{ClientBound, EventQueue, ServerBound, SocketStream, WireCodec};
use std::time::Duration;

/// Serves a [`DevToolsEndpoint`] over one framed local-socket connection.
///
/// Constructed from an accepted [`SocketStream`]
/// (see [`SocketListener::accept`](crate::transport::SocketListener::accept)),
/// it owns the connection for the lifetime of a single client session.
#[derive(Debug)]
pub struct SocketTransport {
    stream: SocketStream,
    served: usize,
}

impl SocketTransport {
    /// Wraps an accepted connection ready to drive an endpoint.
    #[must_use]
    pub fn new(stream: SocketStream) -> Self {
        Self { stream, served: 0 }
    }

    /// Returns how many messages this transport has served so far.
    #[must_use]
    pub const fn served(&self) -> usize {
        self.served
    }

    /// Reads and answers exactly one client message, driving `endpoint`.
    ///
    /// Returns `Ok(false)` when the client has closed the connection cleanly (no
    /// more messages) or the per-connection [`MESSAGE_LIMIT`] has been reached,
    /// and `Ok(true)` after a message was answered. A message that does not
    /// decode is answered with a [`ClientBound::Error`] carrying
    /// [`ProtocolError::MalformedMessage`](obscur_devtools_protocol::error::ProtocolError::MalformedMessage)
    /// rather than terminating the session.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] on any underlying socket read/write failure.
    pub fn serve_one<S: InspectSource>(
        &mut self,
        endpoint: &mut DevToolsEndpoint<S>,
    ) -> io::Result<bool> {
        if self.served >= MESSAGE_LIMIT {
            return Ok(false);
        }
        let Some(frame) = self.stream.read_frame()? else {
            return Ok(false);
        };
        self.served += 1;
        let reply = match WireCodec::decode_server_bound(&frame) {
            Ok(message) => answer(endpoint, message),
            Err(error) => ClientBound::Error(error.into()),
        };
        self.stream
            .write_frame(&WireCodec::encode_client_bound(&reply))?;
        Ok(true)
    }

    /// Drives `endpoint` until the client disconnects or the message limit is
    /// reached, answering each message in turn.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] on any underlying socket read/write failure.
    pub fn serve<S: InspectSource>(
        &mut self,
        endpoint: &mut DevToolsEndpoint<S>,
    ) -> io::Result<()> {
        while self.serve_one(endpoint)? {}
        Ok(())
    }

    /// Drives `endpoint` like [`serve`](Self::serve), additionally **pushing**
    /// queued events to subscribers between client messages (spec §7.6).
    ///
    /// This is what makes the endpoint live rather than poll-only: a host states
    /// what happened by pushing onto `events` from its own thread, and this loop
    /// fans each out through the endpoint — so subscription state, scopes and
    /// authentication stay exactly where they were, on this thread.
    ///
    /// # Why this is what grants a subscription
    ///
    /// Entering this loop is what makes `granted: true` true, so it is this call
    /// — not the host's word for it — that marks the endpoint as servicing event
    /// push. An endpoint driven by [`serve`](Self::serve) instead denies every
    /// `Subscribe`, because nothing there would ever drain a queue.
    ///
    /// # Why a read timeout
    ///
    /// A blocking read parks here until the client speaks, so a queued event could
    /// not be written until the client happened to send something — which is not a
    /// push. `poll` bounds the wait: the loop wakes, drains the queue, and goes
    /// back to reading. It is the *maximum* latency of a push, not a busy-wait —
    /// nothing spins, the thread sleeps in `read` between wakes.
    ///
    /// An event for a topic nobody subscribed to fans out to nothing and writes
    /// nothing, so an unwatched host pays only the wake.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] on any underlying socket read/write failure. A
    /// read that merely timed out is not a failure and does not end the session.
    pub fn serve_with_events<S: InspectSource>(
        &mut self,
        endpoint: &mut DevToolsEndpoint<S>,
        events: &EventQueue,
        poll: Duration,
    ) -> io::Result<()> {
        endpoint.mark_event_push_serviced();
        self.stream.set_read_timeout(Some(poll))?;
        loop {
            match self.serve_one(endpoint) {
                Ok(true) => {}
                // The client hung up or the message limit was reached.
                Ok(false) => return Ok(()),
                Err(error) if is_timeout(&error) => {}
                Err(error) => return Err(error),
            }
            for (topic, kind) in events.drain() {
                // The ENDPOINT decides who hears it — this loop never inspects or
                // bypasses a subscription.
                for event in endpoint.fan_out(topic, &kind) {
                    self.stream
                        .write_frame(&WireCodec::encode_client_bound(&ClientBound::Event(event)))?;
                }
            }
        }
    }
}

/// Whether `error` is a read that simply timed out rather than a broken socket.
///
/// Platforms disagree on which kind a socket read timeout reports, so both are
/// accepted; treating either as fatal would end a healthy session on its first
/// quiet interval.
fn is_timeout(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    )
}

/// Feeds one decoded client message through the endpoint and maps its typed
/// outcome to the matching client-bound reply. The endpoint enforces all §7.7
/// checks; this function only routes the result onto the wire.
fn answer<S: InspectSource>(
    endpoint: &mut DevToolsEndpoint<S>,
    message: ServerBound,
) -> ClientBound {
    match message {
        ServerBound::Handshake(request) => ClientBound::Handshake(endpoint.handshake(&request)),
        ServerBound::Command(command) => match endpoint.dispatch(&command) {
            Ok(CommandOutcome::Snapshot(snapshot)) => ClientBound::Snapshot(snapshot),
            Ok(CommandOutcome::Subscribed(subscription)) => ClientBound::Subscription(subscription),
            Ok(CommandOutcome::Unsubscribed(_) | CommandOutcome::Acknowledged(_)) => {
                ClientBound::Acknowledged(command.id)
            }
            Err(error) => ClientBound::Error(error),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::Authenticator;
    use crate::bridge::{RuntimeBridge, SnapshotInspectSource};
    use crate::transport::{SocketClient, SocketListener};
    use obscur_devtools_protocol::error::ProtocolError;
    use obscur_devtools_protocol::message::{
        CommandId, CommandKind, CommandMessage, ControlAction, HandshakeRequest, Inspectable,
        PermissionScope, ProtocolVersion, ScopeSet, SnapshotPayload, VersionRange,
    };

    fn handshake(token: &str) -> HandshakeRequest {
        HandshakeRequest::new(VersionRange::exact(ProtocolVersion::CURRENT), token)
    }

    /// Spins up a listener on an ephemeral loopback port, serves one endpoint on
    /// a worker thread, and runs `client` against the connected socket.
    fn with_served_endpoint<F>(scopes: ScopeSet, source: SnapshotInspectSource, client: F)
    where
        F: FnOnce(&mut SocketClient),
    {
        let listener = SocketListener::bind_loopback().expect("bind");
        let addr = listener.local_addr().expect("addr").expect("tcp addr");

        let handle = std::thread::spawn(move || {
            let mut auth = Authenticator::new();
            auth.register_token("dev", scopes);
            let mut endpoint = DevToolsEndpoint::new(auth, RuntimeBridge::new(source));
            endpoint.enable();
            let stream = listener.accept().expect("accept");
            let mut transport = SocketTransport::new(stream);
            transport.serve(&mut endpoint).expect("serve");
            endpoint
        });

        let mut connected = SocketClient::connect_loopback(addr).expect("connect");
        client(&mut connected);
        drop(connected);
        handle.join().expect("join");
    }

    #[test]
    fn real_socket_round_trips_handshake_and_inspect() {
        let source = SnapshotInspectSource::new().with_snapshot(
            Inspectable::Tree,
            SnapshotPayload::new("live-tree", 2, vec![7, 8]),
        );
        with_served_endpoint(ScopeSet::all_read(), source, |client| {
            let response = client.handshake(&handshake("dev")).expect("handshake io");
            assert!(response.is_accepted());

            let reply = client
                .command(&CommandMessage::new(
                    CommandId::new(1),
                    CommandKind::Inspect(Inspectable::Tree),
                ))
                .expect("inspect io");
            match reply {
                ClientBound::Snapshot(snapshot) => {
                    assert_eq!(snapshot.target, Inspectable::Tree);
                    assert_eq!(&*snapshot.payload.description, "live-tree");
                    assert_eq!(snapshot.payload.entity_ids, vec![7, 8]);
                }
                other => panic!("expected snapshot, got {other:?}"),
            }
        });
    }

    #[test]
    fn real_socket_rejects_control_without_scope() {
        // Read-only session: a control command must be refused over the socket
        // exactly as the in-process endpoint refuses it.
        with_served_endpoint(
            ScopeSet::all_read(),
            SnapshotInspectSource::new(),
            |client| {
                assert!(
                    client
                        .handshake(&handshake("dev"))
                        .expect("handshake io")
                        .is_accepted()
                );
                let reply = client
                    .command(&CommandMessage::new(
                        CommandId::new(2),
                        CommandKind::Control(ControlAction::Pause),
                    ))
                    .expect("control io");
                assert_eq!(
                    reply,
                    ClientBound::Error(ProtocolError::Unauthorized {
                        required: PermissionScope::Control,
                    })
                );
            },
        );
    }

    #[test]
    fn real_socket_rejects_command_before_handshake() {
        // No handshake ⇒ empty granted scopes ⇒ even a read is unauthorized.
        with_served_endpoint(ScopeSet::all(), SnapshotInspectSource::new(), |client| {
            let reply = client
                .command(&CommandMessage::new(
                    CommandId::new(3),
                    CommandKind::Inspect(Inspectable::Tree),
                ))
                .expect("inspect io");
            assert_eq!(
                reply,
                ClientBound::Error(ProtocolError::Unauthorized {
                    required: PermissionScope::Read,
                })
            );
        });
    }

    #[test]
    fn a_subscriber_receives_a_pushed_event_it_never_asked_for() {
        // THE §7.6 gap this closes: the endpoint was request/reply only. `fan_out`
        // built events and nothing ever wrote one to a socket, so "live inspectable
        // state" was really "polled state".
        use crate::transport::EventQueue;
        use obscur_devtools_protocol::message::{EventKind, Topic};

        let listener = SocketListener::bind_loopback().expect("bind");
        let addr = listener.local_addr().expect("addr").expect("tcp addr");
        let events = EventQueue::new();
        let host_side = events.clone();

        let handle = std::thread::spawn(move || {
            let mut auth = Authenticator::new();
            auth.register_token("dev", ScopeSet::all_read().with(PermissionScope::Subscribe));
            let mut endpoint =
                DevToolsEndpoint::new(auth, RuntimeBridge::new(SnapshotInspectSource::new()));
            endpoint.enable();
            let stream = listener.accept().expect("accept");
            let mut transport = SocketTransport::new(stream);
            transport
                .serve_with_events(&mut endpoint, &events, Duration::from_millis(5))
                .expect("serve");
        });

        let mut client = SocketClient::connect_loopback(addr).expect("connect");
        assert!(
            client
                .handshake(&handshake("dev"))
                .expect("io")
                .is_accepted()
        );
        // Subscribe, so the endpoint knows this client wants frame events.
        let reply = client
            .command(&CommandMessage::new(
                CommandId::new(1),
                CommandKind::Subscribe(Topic::Frames),
            ))
            .expect("subscribe io");
        assert!(matches!(reply, ClientBound::Subscription(_)), "{reply:?}");

        // The HOST states what happened, from its own thread. It never touches the
        // endpoint, so authentication and scopes stay on the serving side.
        assert!(host_side.push(Topic::Frames, EventKind::FrameCommitted { id: 42 }));

        // …and it arrives with no request paired to it.
        client
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("timeout");
        let pushed = client.receive().expect("io").expect("an event was pushed");
        match pushed {
            ClientBound::Event(event) => {
                assert_eq!(event.kind, EventKind::FrameCommitted { id: 42 });
                assert_eq!(event.topic, Topic::Frames);
            }
            other => panic!("expected a pushed event, got {other:?}"),
        }

        drop(client);
        handle.join().expect("join");
    }

    #[test]
    fn an_event_on_an_unsubscribed_topic_is_pushed_to_nobody() {
        // Fan-out stays the endpoint's decision: a host cannot reach a client that
        // did not ask, however loudly it pushes.
        use crate::transport::EventQueue;
        use obscur_devtools_protocol::message::{EventKind, Topic};

        let listener = SocketListener::bind_loopback().expect("bind");
        let addr = listener.local_addr().expect("addr").expect("tcp addr");
        let events = EventQueue::new();
        let host_side = events.clone();

        let handle = std::thread::spawn(move || {
            let mut auth = Authenticator::new();
            auth.register_token("dev", ScopeSet::all_read().with(PermissionScope::Subscribe));
            let mut endpoint =
                DevToolsEndpoint::new(auth, RuntimeBridge::new(SnapshotInspectSource::new()));
            endpoint.enable();
            let stream = listener.accept().expect("accept");
            let mut transport = SocketTransport::new(stream);
            transport
                .serve_with_events(&mut endpoint, &events, Duration::from_millis(5))
                .expect("serve");
        });

        let mut client = SocketClient::connect_loopback(addr).expect("connect");
        assert!(
            client
                .handshake(&handshake("dev"))
                .expect("io")
                .is_accepted()
        );
        // Subscribed to Frames only.
        let _ = client
            .command(&CommandMessage::new(
                CommandId::new(1),
                CommandKind::Subscribe(Topic::Frames),
            ))
            .expect("subscribe io");

        // An event on a DIFFERENT topic.
        assert!(host_side.push(Topic::Diagnostics, EventKind::FrameCommitted { id: 1 }));

        // Nothing arrives: the read times out rather than delivering it.
        client
            .set_read_timeout(Some(Duration::from_millis(150)))
            .expect("timeout");
        let outcome = client.receive();
        assert!(
            outcome.is_err(),
            "an unsubscribed topic must push nothing, got {outcome:?}"
        );

        drop(client);
        handle.join().expect("join");
    }
}
