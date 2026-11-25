//! The `DevToolsEndpoint`: the security-enforcing heart of the server (§7.6/§7.7).

use obscur_devtools_protocol::error::ProtocolError;
use obscur_devtools_protocol::message::{
    CommandId, CommandKind, CommandMessage, ControlAction, HandshakeRejection, HandshakeRequest,
    HandshakeResponse, Inspectable, ProtocolVersion, ScopeSet, SnapshotMessage, Subscription,
    SubscriptionId, Topic, VersionRange,
};

use crate::auth::Authenticator;
use crate::bridge::{InspectSource, RuntimeBridge};
use crate::subscription::SubscriptionManager;
use crate::transport::InjectionSink;

pub use crate::audit_entry::AuditEntry;
pub use crate::command_outcome::CommandOutcome;

#[doc = include_str!("../docs/api/endpoint.doc/DevToolsEndpoint.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevToolsEndpoint<S> {
    authenticator: Authenticator,
    subscriptions: SubscriptionManager,
    bridge: RuntimeBridge<S>,
    supported: VersionRange,
    granted: ScopeSet,
    version: Option<ProtocolVersion>,
    connected: bool,
    enabled: bool,
    control_excluded: bool,
    #[doc = include_str!("../docs/api/endpoint.doc/event_push.md")]
    event_push: bool,
    audit_log: Vec<AuditEntry>,
    #[doc = include_str!("../docs/api/endpoint.doc/injections.md")]
    injections: InjectionSink,
    #[doc = include_str!("../docs/api/endpoint.doc/pending_controls.md")]
    pending_controls: Vec<ControlAction>,
}

impl<S: InspectSource> DevToolsEndpoint<S> {
    #[doc = include_str!("../docs/api/endpoint.doc/new_2.md")]
    pub fn new(authenticator: Authenticator, bridge: RuntimeBridge<S>) -> Self {
        Self {
            authenticator,
            subscriptions: SubscriptionManager::new(),
            bridge,
            supported: VersionRange::exact(ProtocolVersion::CURRENT),
            granted: ScopeSet::empty(),
            version: None,
            connected: false,
            enabled: false,
            control_excluded: false,
            event_push: false,
            audit_log: Vec::new(),
            injections: InjectionSink::disconnected(),
            pending_controls: Vec::new(),
        }
    }

    /// Sets the range of protocol versions this endpoint will negotiate.
    #[must_use]
    pub fn with_supported_versions(mut self, supported: VersionRange) -> Self {
        self.supported = supported;
        self
    }

    #[doc = include_str!("../docs/api/endpoint.doc/with_injection_sink.md")]
    #[must_use]
    pub fn with_injection_sink(mut self, injections: InjectionSink) -> Self {
        self.injections = injections;
        self
    }

    /// Marks this endpoint as control-excluded: `Control` commands are refused
    /// with [`ProtocolError::ControlDisabled`] regardless of granted scope
    /// (spec §7.7).
    #[must_use]
    pub fn with_control_excluded(mut self) -> Self {
        self.control_excluded = true;
        self
    }

    #[doc = include_str!("../docs/api/endpoint.doc/enable.md")]
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    #[doc = include_str!("../docs/api/endpoint.doc/handshake.md")]
    pub fn handshake(&mut self, request: &HandshakeRequest) -> HandshakeResponse {
        if !self.enabled {
            return HandshakeResponse::rejected(HandshakeRejection::Unauthorized);
        }
        let Some(version) = self.supported.negotiate(request.client_versions) else {
            return HandshakeResponse::rejected(HandshakeRejection::VersionUnsupported);
        };
        match self.authenticator.authenticate(&request.token) {
            Some(granted) if !granted.is_empty() => {
                self.granted = granted;
                self.version = Some(version);
                self.connected = true;
                HandshakeResponse::accepted(version, granted)
            }
            _ => HandshakeResponse::rejected(HandshakeRejection::Unauthorized),
        }
    }

    #[doc = include_str!("../docs/api/endpoint.doc/dispatch.md")]
    pub fn dispatch(&mut self, command: &CommandMessage) -> Result<CommandOutcome, ProtocolError> {
        let required = command.required_scope();
        if !self.granted.contains(required) {
            return Err(ProtocolError::Unauthorized { required });
        }
        if command.is_dangerous() {
            self.audit_log
                .push(AuditEntry::new(command.id, command.kind.clone()));
        }
        if self.control_excluded && matches!(command.kind, CommandKind::Control(_)) {
            return Err(ProtocolError::ControlDisabled);
        }
        match &command.kind {
            CommandKind::Inspect(target) => Ok(CommandOutcome::Snapshot(
                self.bridge.inspect(command.id, *target),
            )),
            CommandKind::SelectSemantic { role, name } => Ok(CommandOutcome::Snapshot(
                self.bridge
                    .select_semantic(command.id, role, name.as_deref()),
            )),
            CommandKind::SelectStableId(selector) => Ok(CommandOutcome::Snapshot(
                self.bridge.select_stable(command.id, selector),
            )),
            CommandKind::Subscribe(topic) => {
                // A grant is a promise to deliver. This session is request/reply
                // only unless a pushing loop said otherwise, and a client told
                // `granted: true` by one waits forever for an event that has no
                // path to it — so deny, and let the client know now.
                if !self.event_push {
                    return Ok(CommandOutcome::Subscribed(Subscription::denied(
                        SubscriptionId::default(),
                        *topic,
                    )));
                }
                let id = self.subscriptions.subscribe(*topic)?;
                Ok(CommandOutcome::Subscribed(Subscription::granted(
                    id, *topic,
                )))
            }
            CommandKind::Unsubscribe(id) => Ok(CommandOutcome::Unsubscribed(
                self.subscriptions.unsubscribe(*id),
            )),
            CommandKind::Control(action) => {
                self.pending_controls.push(*action);
                Ok(CommandOutcome::Acknowledged(command.id))
            }
            CommandKind::Inject(event) => {
                // An acknowledgement is a claim that the event happened. It is
                // only true if something received it — so deliver first, and
                // refuse when there is nothing on the other end rather than
                // answering `Acknowledged` to a click that goes nowhere.
                if self.injections.deliver(*event) {
                    Ok(CommandOutcome::Acknowledged(command.id))
                } else {
                    Err(ProtocolError::InjectionUnavailable)
                }
            }
        }
    }

    #[doc = include_str!("../docs/api/endpoint.doc/services_injection.md")]
    #[must_use]
    pub fn services_injection(&self) -> bool {
        self.injections.is_live()
    }

    #[doc = include_str!("../docs/api/endpoint.doc/drain_controls.md")]
    pub fn drain_controls(&mut self) -> Vec<ControlAction> {
        std::mem::take(&mut self.pending_controls)
    }

    /// Convenience: inspect `target` directly through the runtime bridge,
    /// bypassing the command envelope (used by hosts that already authorised).
    pub fn inspect(&self, command: CommandId, target: Inspectable) -> SnapshotMessage {
        self.bridge.inspect(command, target)
    }

    /// Returns the recorded dangerous operations (spec §7.7).
    #[must_use]
    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }

    /// Returns whether a session has successfully handshook.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Returns whether the endpoint has been enabled by the host.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the scopes granted to the current session.
    #[must_use]
    pub fn granted_scopes(&self) -> ScopeSet {
        self.granted
    }

    /// Returns the negotiated protocol version, if the session is connected.
    #[must_use]
    pub fn negotiated_version(&self) -> Option<ProtocolVersion> {
        self.version
    }

    /// Returns a reference to the subscription manager.
    #[must_use]
    pub fn subscriptions(&self) -> &SubscriptionManager {
        &self.subscriptions
    }

    /// Returns whether `Control` commands are refused on this endpoint.
    #[must_use]
    pub fn control_excluded(&self) -> bool {
        self.control_excluded
    }

    #[doc = include_str!("../docs/api/endpoint.doc/services_event_push.md")]
    #[must_use]
    pub fn services_event_push(&self) -> bool {
        self.event_push
    }

    #[doc = include_str!("../docs/api/endpoint.doc/mark_event_push_serviced.md")]
    pub(crate) fn mark_event_push_serviced(&mut self) {
        self.event_push = true;
    }

    /// Builds the events to push to subscribers of `topic` for `kind`.
    #[must_use]
    pub fn fan_out(
        &self,
        topic: Topic,
        kind: &obscur_devtools_protocol::message::EventKind,
    ) -> Vec<obscur_devtools_protocol::message::ProtocolEvent> {
        self.subscriptions.fan_out(topic, kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::NullInspectSource;
    use crate::transport::InjectionQueue;
    use obscur_devtools_protocol::message::{
        ControlAction, EventKind, InjectedEvent, PermissionScope, SnapshotPayload,
    };

    /// A test source returning a payload tagged with the inspected target.
    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    struct FixedSource;

    impl InspectSource for FixedSource {
        fn snapshot(&self, target: Inspectable) -> SnapshotPayload {
            SnapshotPayload::new(format!("state:{target:?}"), 1, vec![42])
        }
    }

    fn endpoint_with(scopes: ScopeSet, token: &str) -> DevToolsEndpoint<FixedSource> {
        let mut auth = Authenticator::new();
        auth.register_token(token, scopes);
        let mut ep = DevToolsEndpoint::new(auth, RuntimeBridge::new(FixedSource));
        ep.enable();
        ep
    }

    fn handshake(token: &str) -> HandshakeRequest {
        HandshakeRequest::new(VersionRange::exact(ProtocolVersion::CURRENT), token)
    }

    #[test]
    fn disabled_endpoint_rejects_handshake() {
        let mut auth = Authenticator::new();
        auth.register_token("dev", ScopeSet::all());
        let mut ep = DevToolsEndpoint::new(auth, RuntimeBridge::new(NullInspectSource));
        assert!(!ep.is_enabled());
        let resp = ep.handshake(&handshake("dev"));
        assert_eq!(
            resp,
            HandshakeResponse::rejected(HandshakeRejection::Unauthorized)
        );
        assert!(!ep.is_connected());
    }

    #[test]
    fn handshake_accepts_valid_token_and_version() {
        let mut ep = endpoint_with(ScopeSet::observer(), "dev");
        let resp = ep.handshake(&handshake("dev"));
        assert_eq!(
            resp,
            HandshakeResponse::accepted(ProtocolVersion::CURRENT, ScopeSet::observer())
        );
        assert!(ep.is_connected());
        assert_eq!(ep.negotiated_version(), Some(ProtocolVersion::CURRENT));
        assert_eq!(ep.granted_scopes(), ScopeSet::observer());
    }

    #[test]
    fn handshake_rejects_incompatible_version() {
        let mut ep = endpoint_with(ScopeSet::all(), "dev");
        let request = HandshakeRequest::new(VersionRange::exact(ProtocolVersion::new(2, 0)), "dev");
        assert_eq!(
            ep.handshake(&request),
            HandshakeResponse::rejected(HandshakeRejection::VersionUnsupported)
        );
        assert!(!ep.is_connected());
    }

    #[test]
    fn handshake_rejects_unknown_token() {
        let mut ep = endpoint_with(ScopeSet::all(), "dev");
        assert_eq!(
            ep.handshake(&handshake("intruder")),
            HandshakeResponse::rejected(HandshakeRejection::Unauthorized)
        );
        assert!(!ep.is_connected());
    }

    #[test]
    fn read_only_session_can_inspect() {
        let mut ep = endpoint_with(ScopeSet::all_read(), "dev");
        ep.handshake(&handshake("dev"));
        let cmd = CommandMessage::new(CommandId::new(1), CommandKind::Inspect(Inspectable::Tree));
        let outcome = ep.dispatch(&cmd).expect("read is authorised");
        match outcome {
            CommandOutcome::Snapshot(msg) => {
                assert_eq!(msg.target, Inspectable::Tree);
                assert_eq!(&*msg.payload.description, "state:Tree");
            }
            other => panic!("expected snapshot, got {other:?}"),
        }
    }

    #[test]
    fn read_only_session_cannot_control() {
        let mut ep = endpoint_with(ScopeSet::all_read(), "dev");
        ep.handshake(&handshake("dev"));
        let cmd = CommandMessage::new(
            CommandId::new(2),
            CommandKind::Control(ControlAction::Pause),
        );
        assert_eq!(
            ep.dispatch(&cmd),
            Err(ProtocolError::Unauthorized {
                required: PermissionScope::Control
            })
        );
        assert!(ep.audit_log().is_empty());
    }

    #[test]
    fn control_granted_session_is_audited_and_acknowledged() {
        let mut ep = endpoint_with(ScopeSet::all(), "dev");
        ep.handshake(&handshake("dev"));
        let cmd = CommandMessage::new(
            CommandId::new(3),
            CommandKind::Control(ControlAction::StepFrame),
        );
        let outcome = ep.dispatch(&cmd).expect("control is authorised");
        assert_eq!(outcome, CommandOutcome::Acknowledged(CommandId::new(3)));
        assert_eq!(ep.audit_log().len(), 1);
        assert_eq!(ep.audit_log()[0].command, CommandId::new(3));
    }

    #[test]
    fn control_excluded_endpoint_refuses_control() {
        let mut auth = Authenticator::new();
        auth.register_token("dev", ScopeSet::all());
        let mut ep =
            DevToolsEndpoint::new(auth, RuntimeBridge::new(FixedSource)).with_control_excluded();
        ep.enable();
        ep.handshake(&handshake("dev"));
        assert!(ep.control_excluded());
        let cmd = CommandMessage::new(
            CommandId::new(4),
            CommandKind::Control(ControlAction::Resume),
        );
        assert_eq!(ep.dispatch(&cmd), Err(ProtocolError::ControlDisabled));
    }

    #[test]
    fn unconnected_session_is_unauthorized() {
        let mut ep = endpoint_with(ScopeSet::all(), "dev");
        // No handshake performed: granted scopes are empty.
        let cmd = CommandMessage::new(CommandId::new(5), CommandKind::Inspect(Inspectable::Tree));
        assert_eq!(
            ep.dispatch(&cmd),
            Err(ProtocolError::Unauthorized {
                required: PermissionScope::Read
            })
        );
    }

    #[test]
    fn event_push_never_loosens_the_subscribe_scope_check() {
        // The §7.7 boundary must be unmoved by the push work: a session without
        // the Subscribe scope is refused on the scope, before the grant question
        // is ever reached — whether or not anyone is pushing.
        for pushing in [false, true] {
            let mut ep = endpoint_with(ScopeSet::all_read(), "dev");
            if pushing {
                ep.mark_event_push_serviced();
            }
            ep.handshake(&handshake("dev"));
            assert_eq!(
                ep.dispatch(&CommandMessage::new(
                    CommandId::new(1),
                    CommandKind::Subscribe(Topic::Frames),
                )),
                Err(ProtocolError::Unauthorized {
                    required: PermissionScope::Subscribe
                }),
                "pushing={pushing}"
            );
        }
    }

    #[test]
    fn a_subscribe_is_denied_when_nothing_will_push() {
        // The §7-C lie: an endpoint nobody pushes through used to answer
        // `granted: true` and leave the client waiting forever.
        let mut ep = endpoint_with(ScopeSet::all(), "dev");
        ep.handshake(&handshake("dev"));
        assert!(!ep.services_event_push());

        let outcome = ep
            .dispatch(&CommandMessage::new(
                CommandId::new(1),
                CommandKind::Subscribe(Topic::Frames),
            ))
            .expect("subscribe is authorised — it is refused on the merits, not the scope");
        match outcome {
            CommandOutcome::Subscribed(subscription) => {
                assert!(!subscription.granted, "denied, not silently accepted");
                assert_eq!(subscription.topic, Topic::Frames);
            }
            other => panic!("expected a subscription record, got {other:?}"),
        }
        // And nothing was registered, so no fan-out can pretend otherwise.
        assert!(ep.subscriptions().is_empty());
        assert!(
            ep.fan_out(Topic::Frames, &EventKind::FrameCommitted { id: 1 })
                .is_empty()
        );
    }

    #[test]
    fn an_inject_is_refused_when_no_host_can_receive_it() {
        // The §7-C lie, the other way round from `Subscribe`: an endpoint with
        // nowhere to put a synthetic click used to answer `Acknowledged` and
        // drop it. It is now refused — and refused for the right reason, so the
        // client is not sent looking for a token it already has.
        let mut ep = endpoint_with(ScopeSet::all(), "dev");
        ep.handshake(&handshake("dev"));
        assert!(!ep.services_injection());
        assert_eq!(
            ep.dispatch(&CommandMessage::new(
                CommandId::new(1),
                CommandKind::Inject(InjectedEvent::PointerDown { x: 4, y: 5 }),
            )),
            Err(ProtocolError::InjectionUnavailable)
        );
    }

    #[test]
    fn an_injection_that_cannot_be_serviced_is_still_audited() {
        // §7.7 records the attempt, not the outcome: a refusal must not be a way
        // to inject without leaving a trace. (`ControlDisabled` audits the same
        // way, and this keeps the two consistent.)
        let mut ep = endpoint_with(ScopeSet::all(), "dev");
        ep.handshake(&handshake("dev"));
        let _ = ep.dispatch(&CommandMessage::new(
            CommandId::new(9),
            CommandKind::Inject(InjectedEvent::KeyDown { key: 65 }),
        ));
        assert_eq!(ep.audit_log().len(), 1);
        assert_eq!(ep.audit_log()[0].command, CommandId::new(9));
    }

    #[test]
    fn an_authorized_inject_reaches_the_hosts_queue() {
        // The positive half: with a host on the other end the acknowledgement is
        // true — the event is in the host's queue, on the host's thread.
        let queue = InjectionQueue::new();
        let mut auth = Authenticator::new();
        auth.register_token("dev", ScopeSet::all());
        let mut ep = DevToolsEndpoint::new(auth, RuntimeBridge::new(FixedSource))
            .with_injection_sink(queue.sink());
        ep.enable();
        ep.handshake(&handshake("dev"));
        assert!(ep.services_injection());

        assert_eq!(
            ep.dispatch(&CommandMessage::new(
                CommandId::new(1),
                CommandKind::Inject(InjectedEvent::PointerDown { x: 4, y: 5 }),
            )),
            Ok(CommandOutcome::Acknowledged(CommandId::new(1)))
        );
        assert_eq!(
            queue.drain(),
            vec![InjectedEvent::PointerDown { x: 4, y: 5 }],
            "the acknowledged event is where the host will look for it"
        );
    }

    #[test]
    fn an_endpoint_outliving_its_hosts_queue_refuses_again() {
        // A sink is a `Weak`, so "someone can receive this" is re-checked on
        // every command rather than remembered from construction.
        let queue = InjectionQueue::new();
        let mut auth = Authenticator::new();
        auth.register_token("dev", ScopeSet::all());
        let mut ep = DevToolsEndpoint::new(auth, RuntimeBridge::new(FixedSource))
            .with_injection_sink(queue.sink());
        ep.enable();
        ep.handshake(&handshake("dev"));
        assert!(ep.services_injection());
        drop(queue);
        assert!(!ep.services_injection());
        assert_eq!(
            ep.dispatch(&CommandMessage::new(
                CommandId::new(2),
                CommandKind::Inject(InjectedEvent::PointerUp { x: 0, y: 0 }),
            )),
            Err(ProtocolError::InjectionUnavailable)
        );
    }

    #[test]
    fn an_injection_sink_never_loosens_the_inject_scope_check() {
        // The §7.7 boundary must be unmoved by this work: a session without the
        // Inject scope is refused ON THE SCOPE, before the destination question
        // is ever asked — whether or not a host is listening. An injected event
        // respects exactly the scopes a read does.
        for wired in [false, true] {
            let queue = InjectionQueue::new();
            let mut auth = Authenticator::new();
            auth.register_token("dev", ScopeSet::all_read());
            let mut ep = DevToolsEndpoint::new(auth, RuntimeBridge::new(FixedSource));
            if wired {
                ep = ep.with_injection_sink(queue.sink());
            }
            ep.enable();
            ep.handshake(&handshake("dev"));
            assert_eq!(
                ep.dispatch(&CommandMessage::new(
                    CommandId::new(1),
                    CommandKind::Inject(InjectedEvent::PointerDown { x: 1, y: 1 }),
                )),
                Err(ProtocolError::Unauthorized {
                    required: PermissionScope::Inject
                }),
                "wired={wired}"
            );
            assert!(
                queue.is_empty(),
                "an unauthorized injection reaches no host, wired={wired}"
            );
        }
    }

    #[test]
    fn a_production_authenticator_still_beats_an_installed_sink() {
        // Defence in depth: a host that wires a sink cannot re-open injection on
        // a production build — the scope was stripped at authentication, which is
        // upstream of everything here.
        let queue = InjectionQueue::new();
        let mut auth = Authenticator::production();
        auth.register_token("dev", ScopeSet::all());
        let mut ep = DevToolsEndpoint::new(auth, RuntimeBridge::new(FixedSource))
            .with_injection_sink(queue.sink());
        ep.enable();
        ep.handshake(&handshake("dev"));
        assert!(!ep.granted_scopes().contains(PermissionScope::Inject));
        assert_eq!(
            ep.dispatch(&CommandMessage::new(
                CommandId::new(1),
                CommandKind::Inject(InjectedEvent::PointerDown { x: 1, y: 1 }),
            )),
            Err(ProtocolError::Unauthorized {
                required: PermissionScope::Inject
            })
        );
        assert!(queue.is_empty());
    }

    #[test]
    fn end_to_end_handshake_subscribe_inspect_and_audit() {
        let queue = InjectionQueue::new();
        let mut auth = Authenticator::new();
        auth.register_token("dev", ScopeSet::all());
        let mut ep = DevToolsEndpoint::new(auth, RuntimeBridge::new(FixedSource))
            .with_injection_sink(queue.sink());
        ep.enable();
        // Stand in for the pushing serving loop, which is what makes a grant
        // honest (see `SocketTransport::serve_with_events`).
        ep.mark_event_push_serviced();

        assert!(ep.handshake(&handshake("dev")).is_accepted());

        let sub = ep
            .dispatch(&CommandMessage::new(
                CommandId::new(10),
                CommandKind::Subscribe(Topic::Frames),
            ))
            .expect("subscribe authorised");
        let subscription = match sub {
            CommandOutcome::Subscribed(s) => s,
            other => panic!("expected subscription, got {other:?}"),
        };
        assert!(subscription.granted);

        let events = ep.fan_out(Topic::Frames, &EventKind::FrameCommitted { id: 1 });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].subscription, subscription.id);

        let snapshot = ep
            .dispatch(&CommandMessage::new(
                CommandId::new(11),
                CommandKind::Inspect(Inspectable::Scene),
            ))
            .expect("inspect authorised");
        assert!(matches!(snapshot, CommandOutcome::Snapshot(_)));

        let inject = ep
            .dispatch(&CommandMessage::new(
                CommandId::new(12),
                CommandKind::Inject(InjectedEvent::KeyDown { key: 65 }),
            ))
            .expect("inject authorised");
        assert_eq!(inject, CommandOutcome::Acknowledged(CommandId::new(12)));
        assert_eq!(ep.audit_log().len(), 1);
        assert_eq!(ep.audit_log()[0].command, CommandId::new(12));
    }
}
