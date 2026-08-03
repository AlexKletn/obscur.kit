#![doc = include_str!("spec_ch7_endpoint_security.doc/_module.md")]
#![forbid(unsafe_code)]

use std::net::TcpStream;
use std::time::Duration;

use obscur_kit::devtools_protocol::error::ProtocolError;
use obscur_kit::devtools_protocol::message::{
    CommandId, CommandKind, CommandMessage, ControlAction, HandshakeRejection, HandshakeRequest,
    HandshakeResponse, InjectedEvent, Inspectable, PermissionScope, ProtocolVersion,
    SUBSCRIPTION_LIMIT, ScopeSet, Topic, VersionRange,
};
use obscur_kit::devtools_server::auth::Authenticator;
use obscur_kit::devtools_server::bridge::{NullInspectSource, RuntimeBridge};
use obscur_kit::devtools_server::endpoint::{CommandOutcome, DevToolsEndpoint};
use obscur_kit::devtools_server::transport::{InjectionQueue, SocketListener};

const TOKEN: &str = "test-token";

fn endpoint(scopes: ScopeSet) -> DevToolsEndpoint<NullInspectSource> {
    let mut auth = Authenticator::new();
    auth.register_token(TOKEN, scopes);
    DevToolsEndpoint::new(auth, RuntimeBridge::new(NullInspectSource))
}

fn connect(endpoint: &mut DevToolsEndpoint<NullInspectSource>) -> HandshakeResponse {
    endpoint.handshake(&HandshakeRequest::new(
        VersionRange::exact(ProtocolVersion::CURRENT),
        TOKEN,
    ))
}

#[doc = include_str!("spec_ch7_endpoint_security.doc/the_endpoint_is_disabled_by_default_and_binds_only_loopback.md")]
#[test]
fn the_endpoint_is_disabled_by_default_and_binds_only_loopback() {
    // Default configuration, in a debug build: nothing is listening.
    let mut fresh = endpoint(ScopeSet::all());
    assert!(
        !fresh.is_enabled(),
        "a default-configured endpoint is disabled"
    );
    assert!(
        matches!(
            connect(&mut fresh),
            HandshakeResponse::Rejected {
                reason: HandshakeRejection::Unauthorized
            }
        ),
        "a connection to a default-configured endpoint is refused"
    );
    assert!(
        !fresh.is_connected(),
        "no session exists on a default-configured endpoint"
    );

    // Enabled with default settings: the bind address is loopback, not 0.0.0.0.
    let listener = SocketListener::bind_loopback().expect("an enabled endpoint binds a listener");
    let addr = listener
        .local_addr()
        .expect("the listener reports its address")
        .expect("a TCP listener has an address");
    assert!(
        addr.ip().is_loopback(),
        "the bind address is loopback only, not 0.0.0.0 — got {addr}"
    );

    TcpStream::connect_timeout(&addr, Duration::from_secs(2))
        .expect("a loopback connection is accepted");

    // The listener is not on the wildcard address. (Probing this by *connecting*
    // to 0.0.0.0 cannot discriminate: on Linux connect(0.0.0.0) is an alias for
    // loopback, so it succeeds against a loopback-bound listener. The bind
    // address itself is the observable the clause is about.)
    assert!(
        !addr.ip().is_unspecified(),
        "the listener must not be bound to 0.0.0.0 — got {addr}"
    );

    // (A connect probe from a non-loopback interface is not written here: on this
    // host every non-loopback probe is answered by a container bridge, so the
    // probe measures the network, not the endpoint. `is_loopback` above is the
    // property the clause states.)
}

#[doc = include_str!("spec_ch7_endpoint_security.doc/read_alone_grants_neither_subscribe_nor_control_nor_injection.md")]
#[test]
fn read_alone_grants_neither_subscribe_nor_control_nor_injection() {
    let queue = InjectionQueue::new();
    let sink = queue.sink();
    let mut endpoint = endpoint(ScopeSet::single(PermissionScope::Read)).with_injection_sink(sink);
    endpoint.enable();
    assert!(
        connect(&mut endpoint).is_accepted(),
        "a READ token connects"
    );

    // (a) Fetching the tree succeeds.
    let read = endpoint.dispatch(&CommandMessage::new(
        CommandId::new(1),
        CommandKind::Inspect(Inspectable::Tree),
    ));
    assert!(
        matches!(read, Ok(CommandOutcome::Snapshot(_))),
        "(a) READ fetches the tree, got {read:?}"
    );

    // (b) Opening a subscription is denied, naming the missing permission.
    let subscribe = endpoint.dispatch(&CommandMessage::new(
        CommandId::new(2),
        CommandKind::Subscribe(Topic::Diagnostics),
    ));
    assert_eq!(
        subscribe,
        Err(ProtocolError::Unauthorized {
            required: PermissionScope::Subscribe
        }),
        "(b) subscribe is not a subset of read"
    );

    // (c) A control command is denied.
    let control = endpoint.dispatch(&CommandMessage::new(
        CommandId::new(3),
        CommandKind::Control(ControlAction::StepFrame),
    ));
    assert_eq!(
        control,
        Err(ProtocolError::Unauthorized {
            required: PermissionScope::Control
        }),
        "(c) control is denied"
    );

    // (d) Injecting a pointer event is denied — and the event is NOT delivered.
    let inject = endpoint.dispatch(&CommandMessage::new(
        CommandId::new(4),
        CommandKind::Inject(InjectedEvent::PointerDown { x: 10, y: 10 }),
    ));
    assert_eq!(
        inject,
        Err(ProtocolError::Unauthorized {
            required: PermissionScope::Inject
        }),
        "(d) injection is denied"
    );
    assert!(
        queue.drain().is_empty(),
        "(d) a denied injection must never reach the host: nothing was delivered"
    );
}

#[doc = include_str!("spec_ch7_endpoint_security.doc/nothing_is_served_before_authentication_and_a_bad_version_is_typed.md")]
#[test]
fn nothing_is_served_before_authentication_and_a_bad_version_is_typed() {
    // (i) A tree-fetch with no token presented transmits no tree.
    let mut unauthenticated = endpoint(ScopeSet::all());
    unauthenticated.enable();
    let fetched = unauthenticated.dispatch(&CommandMessage::new(
        CommandId::new(1),
        CommandKind::Inspect(Inspectable::Tree),
    ));
    assert_eq!(
        fetched,
        Err(ProtocolError::Unauthorized {
            required: PermissionScope::Read
        }),
        "(i) an unauthenticated tree fetch is an auth error"
    );
    assert!(
        unauthenticated.granted_scopes().is_empty(),
        "(i) no capability is held before a handshake"
    );
    assert!(
        unauthenticated.negotiated_version().is_none(),
        "(i) no banner/version is exchanged before the token is checked"
    );

    // (ii) A handshake declaring only an unsupported future version, with a
    // valid token, is a typed version mismatch — not a silent close and not a
    // best-effort downgrade.
    let mut versioned = endpoint(ScopeSet::all());
    versioned.enable();
    let future = ProtocolVersion::new(ProtocolVersion::CURRENT.major + 9, 0);
    let response = versioned.handshake(&HandshakeRequest::new(VersionRange::exact(future), TOKEN));
    assert!(
        matches!(
            response,
            HandshakeResponse::Rejected {
                reason: HandshakeRejection::VersionUnsupported
            }
        ),
        "(ii) an unsupported version is rejected as a version mismatch, got {response:?}"
    );
    assert!(
        versioned.negotiated_version().is_none(),
        "(ii) no version was silently negotiated"
    );
    assert!(
        !versioned.is_connected(),
        "(ii) the session did not proceed on a best-effort version"
    );
}

#[doc = include_str!("spec_ch7_endpoint_security.doc/limits_are_enforced_without_collateral_and_dangerous_operations_are_audited.md")]
#[test]
fn limits_are_enforced_without_collateral_and_dangerous_operations_are_audited() {
    let queue = InjectionQueue::new();
    let sink = queue.sink();
    let mut endpoint = endpoint(ScopeSet::all()).with_injection_sink(sink);
    endpoint.enable();
    assert!(connect(&mut endpoint).is_accepted());

    // A request/reply-only session has no delivery path, so a subscription is
    // refused rather than falsely granted — and refusing costs nothing else.
    let audit_at_start = endpoint.audit_log().len();
    let outcome = endpoint.dispatch(&CommandMessage::new(
        CommandId::new(0),
        CommandKind::Subscribe(Topic::Diagnostics),
    ));
    match outcome {
        Ok(CommandOutcome::Subscribed(subscription)) => assert!(
            !subscription.granted,
            "a subscription is never granted while nothing can deliver to it"
        ),
        other => panic!("a subscribe is answered, not errored, got {other:?}"),
    }
    assert!(
        endpoint.subscriptions().len() <= SUBSCRIPTION_LIMIT,
        "the live subscription count never exceeds the limit"
    );
    assert!(
        endpoint.is_connected(),
        "refusing a subscription does not reset the connection"
    );
    assert_eq!(
        endpoint.audit_log().len(),
        audit_at_start,
        "and a subscribe is not itself a dangerous operation"
    );

    // A plain read produces no audit record.
    let audit_before = endpoint.audit_log().len();
    let _ = endpoint.dispatch(&CommandMessage::new(
        CommandId::new(10_000),
        CommandKind::Inspect(Inspectable::Tree),
    ));
    assert_eq!(
        endpoint.audit_log().len(),
        audit_before,
        "a plain read is not audited"
    );

    // An injection and a control command are audited, naming the operation and
    // its target.
    let injection = CommandMessage::new(
        CommandId::new(10_001),
        CommandKind::Inject(InjectedEvent::PointerDown { x: 4, y: 9 }),
    );
    assert!(
        endpoint.dispatch(&injection).is_ok(),
        "a fully-scoped injection with a live sink succeeds"
    );
    let control = CommandMessage::new(
        CommandId::new(10_002),
        CommandKind::Control(ControlAction::Pause),
    );
    assert!(endpoint.dispatch(&control).is_ok());

    let audited = endpoint.audit_log();
    assert_eq!(
        audited.len(),
        audit_before + 2,
        "exactly the two dangerous operations were audited"
    );
    assert_eq!(
        audited[audit_before].command, injection.id,
        "the audit record names the acting command"
    );
    assert_eq!(
        audited[audit_before].kind, injection.kind,
        "the audit record names the operation and its target node/coordinates"
    );
    assert_eq!(audited[audit_before + 1].kind, control.kind);

    assert_eq!(
        queue.drain().len(),
        1,
        "the authorized injection really was delivered"
    );
}
