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

/// A recorded dangerous operation kept in the endpoint's audit log (spec §7.7).
///
/// Every `Control`/`Inject` command that reaches dispatch is appended here so
/// the host has a durable record of who steered or injected into the runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    /// The id of the command that was audited.
    pub command: CommandId,
    /// The dangerous command body that was recorded.
    pub kind: CommandKind,
}

impl AuditEntry {
    /// Creates an audit entry.
    #[must_use]
    pub const fn new(command: CommandId, kind: CommandKind) -> Self {
        Self { command, kind }
    }
}

/// The result of successfully dispatching a [`CommandMessage`].
///
/// Every variant is a typed outcome — the endpoint never executes arbitrary
/// code; control and injection resolve to a plain acknowledgement (spec §7.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    /// An `Inspect` produced a snapshot of live state.
    Snapshot(SnapshotMessage),
    /// A `Subscribe` produced a subscription record.
    Subscribed(Subscription),
    /// An `Unsubscribe` reported whether a subscription was active.
    Unsubscribed(bool),
    /// A `Control` or `Inject` command was applied, keyed by its id.
    Acknowledged(CommandId),
}

/// A single authenticated DevTools session and the security policy around it.
///
/// The endpoint ties the [`Authenticator`], [`SubscriptionManager`], and
/// [`RuntimeBridge`] together and is the one place that enforces the §7.7
/// endpoint-security rules:
///
/// * **Disabled by default** — a fresh endpoint is not [`enable`](Self::enable)d
///   and every handshake is rejected until the host turns it on.
/// * **Local-only, authenticated** — every session presents a token that the
///   [`Authenticator`] must recognise; an unknown token is rejected.
/// * **Version-negotiated** — the server's supported [`VersionRange`] is
///   negotiated against the client's; no overlap ⇒ rejection.
/// * **Scope-separated** — each command is authorised against the granted
///   [`ScopeSet`] using its own required scope, so the four capability classes
///   are independent.
/// * **Limit-enforcing** — subscriptions are bounded by the manager's limit.
/// * **Grant-honest** — a `Subscribe` is granted only when this session is
///   actually being driven by a loop that pushes events (see
///   [`services_event_push`](Self::services_event_push)); otherwise it is
///   answered with a *denied* [`Subscription`] rather than a grant nothing
///   will service. An `Inject` is the same rule in the other direction: it is
///   [`ProtocolError::InjectionUnavailable`] unless the host installed a live
///   [`InjectionSink`] to receive it (see
///   [`with_injection_sink`](Self::with_injection_sink)).
/// * **Audited** — dangerous (`Control`/`Inject`) commands are appended to the
///   audit log before they take effect — and before they are refused, so an
///   attempt is recorded whether or not the host could service it.
/// * **No arbitrary execution** — control/injection resolve to typed
///   acknowledgements only.
///
/// A production endpoint may be built [`with_control_excluded`](Self::with_control_excluded),
/// which turns every `Control` command into a [`ProtocolError::ControlDisabled`].
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
    /// Whether a loop that actually drains an [`EventQueue`] and writes the
    /// fan-out is driving this session. Only such a loop sets it — see
    /// [`mark_event_push_serviced`](Self::mark_event_push_serviced) — so it can
    /// never claim more than is true, and a `Subscribe` is denied while it is
    /// `false`.
    ///
    /// [`EventQueue`]: crate::transport::EventQueue
    event_push: bool,
    audit_log: Vec<AuditEntry>,
    /// Where an authorized synthetic input event is delivered. The endpoint never
    /// touches the runtime itself (spec §7.7): the host owns the queue behind
    /// this sink and applies what it drains (host-provides-device).
    ///
    /// [`disconnected`](InjectionSink::disconnected) — the default — is a host
    /// that cannot receive injected input, and every `Inject` is refused while it
    /// is. That is the §7-C fix: this used to be a `Vec<InjectedEvent>` the
    /// endpoint owned, which meant the events were unreachable from the *thread*
    /// that owns the runtime, so they were acknowledged and then dropped.
    injections: InjectionSink,
    /// Authorized control actions (pause/step/inspect-mode) awaiting the host.
    ///
    /// Still the shape `injections` had, and still with no production drainer:
    /// [`drain_controls`](Self::drain_controls) has no caller in this workspace.
    /// `Control` was outside the fix that inverted the injection channel, and
    /// carrying it unfixed is more honest than implying otherwise.
    pending_controls: Vec<ControlAction>,
}

impl<S: InspectSource> DevToolsEndpoint<S> {
    /// Creates a disabled endpoint from an authenticator and a runtime bridge.
    ///
    /// The endpoint supports exactly the current protocol version until
    /// [`with_supported_versions`](Self::with_supported_versions) widens it, and
    /// must be [`enable`](Self::enable)d before it will accept a handshake.
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

    /// Gives this endpoint somewhere to deliver an authorized `Inject`: the push
    /// end of an [`InjectionQueue`](crate::transport::InjectionQueue) the host
    /// holds and drains (spec §7.6).
    ///
    /// Without one, `Inject` is refused with
    /// [`ProtocolError::InjectionUnavailable`] — see [`dispatch`](Self::dispatch).
    ///
    /// # This grants nothing
    ///
    /// A sink is a *destination*, not a permission. The `Inject` scope check runs
    /// first and is untouched by this: a session without the scope is refused on
    /// the scope whether or not a sink exists, and installing a sink can only ever
    /// move an authorized `Inject` from "refused" to "delivered" — never the
    /// reverse, and never past the [`Authenticator`], which in a
    /// [`production`](Authenticator::production) build has already stripped
    /// `Inject` from every grant (§7.7).
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

    /// Enables the endpoint so it will accept a handshake.
    ///
    /// Endpoints are disabled by default (spec §7.7); this is the explicit
    /// opt-in the host performs.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Performs the connection handshake: version negotiation *and* token
    /// authentication.
    ///
    /// On success the granted scopes and negotiated version are stored, the
    /// session is marked connected, and an [`HandshakeResponse::Accepted`] is
    /// returned. A disabled endpoint, an incompatible version range, or an
    /// unrecognised/empty-scope token each yields an
    /// [`HandshakeResponse::Rejected`].
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

    /// Dispatches an authenticated command, enforcing the §7.7 security core.
    ///
    /// The steps, in order: authorise the command's
    /// [`required_scope`](CommandMessage::required_scope) against the granted
    /// [`ScopeSet`] (a session with no scopes — including one that never
    /// handshook — fails here); audit the command if it is dangerous; refuse
    /// `Control` when control is excluded; then route to the bridge, the
    /// subscription manager, or a typed acknowledgement.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Unauthorized`] when the granted scopes lack the
    /// required scope, [`ProtocolError::ControlDisabled`] when a control command
    /// is issued to a control-excluded endpoint,
    /// [`ProtocolError::InjectionUnavailable`] when an authorized `Inject` has no
    /// live [`InjectionSink`] to be delivered to, and
    /// [`ProtocolError::LimitExceeded`] when a subscription would exceed the
    /// limit.
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

    /// Whether an authorized `Inject` would actually reach a host — i.e. whether
    /// a live [`InjectionSink`] is installed (see
    /// [`with_injection_sink`](Self::with_injection_sink)).
    ///
    /// False on a fresh endpoint and false again once the host drops its
    /// [`InjectionQueue`](crate::transport::InjectionQueue), which is what makes
    /// this a fact about reachability rather than a host's promise.
    #[must_use]
    pub fn services_injection(&self) -> bool {
        self.injections.is_live()
    }

    /// Removes and returns the control actions queued by authorized `Control`
    /// commands since the last drain, in arrival order.
    ///
    /// No production code calls this — see [`pending_controls`](Self#structfield.pending_controls).
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

    /// Whether this session will actually deliver pushed events — and therefore
    /// whether a `Subscribe` is granted rather than denied.
    ///
    /// False until a serving loop that pushes takes the endpoint over, which is
    /// today only
    /// [`SocketTransport::serve_with_events`](crate::transport::SocketTransport::serve_with_events).
    /// A session driven by [`SocketTransport::serve`](crate::transport::SocketTransport::serve)
    /// stays request/reply and reports `false`.
    #[must_use]
    pub fn services_event_push(&self) -> bool {
        self.event_push
    }

    /// Declares that the caller's loop drains an
    /// [`EventQueue`](crate::transport::EventQueue) and writes this endpoint's
    /// [`fan_out`](Self::fan_out) to the client, so subscriptions may be granted.
    ///
    /// Deliberately crate-visible: the promise that `granted: true` makes is
    /// only truthful if the code holding the endpoint really does push, and a
    /// host outside this crate cannot push without going through a transport
    /// here. That keeps the flag a fact rather than a claim.
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

        // Handshake.
        assert!(ep.handshake(&handshake("dev")).is_accepted());

        // Subscribe.
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

        // Fan-out reaches the subscriber.
        let events = ep.fan_out(Topic::Frames, &EventKind::FrameCommitted { id: 1 });
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].subscription, subscription.id);

        // Inspect.
        let snapshot = ep
            .dispatch(&CommandMessage::new(
                CommandId::new(11),
                CommandKind::Inspect(Inspectable::Scene),
            ))
            .expect("inspect authorised");
        assert!(matches!(snapshot, CommandOutcome::Snapshot(_)));

        // Dangerous inject is audited.
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
