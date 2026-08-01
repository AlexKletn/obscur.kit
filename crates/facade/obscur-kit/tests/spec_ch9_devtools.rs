#![doc = include_str!("spec_ch9_devtools.doc/_module.md")]
#![forbid(unsafe_code)]

#[path = "spec_ch9_support/mod.rs"]
mod support;

use obscur_kit::devtools_protocol::error::ProtocolError;
use obscur_kit::devtools_protocol::message::{
    CommandId, CommandKind, CommandMessage, ControlAction, InjectedEvent, Inspectable,
    PermissionScope, ScopeSet,
};
use obscur_kit::devtools_protocol::message::{HandshakeRequest, ProtocolVersion, VersionRange};
use obscur_kit::devtools_server::auth::Authenticator;
use obscur_kit::devtools_server::endpoint::DevToolsEndpoint;
use obscur_kit::devtools_server::transport::SocketListener;
use support::{approx, at, compile, drive, el, geom, view};

const VW: f32 = 1000.0;

#[doc = include_str!("spec_ch9_devtools.doc/client_versions.md")]
fn client_versions() -> VersionRange {
    VersionRange::new(
        ProtocolVersion::new(0, 0),
        ProtocolVersion::new(u16::MAX, u16::MAX),
    )
}

#[doc = include_str!("spec_ch9_devtools.doc/the_diagnostics_endpoint_is_disabled_by_default_and_binds_loopback_when_enabled.md")]
#[test]
fn the_diagnostics_endpoint_is_disabled_by_default_and_binds_loopback_when_enabled() {
    // Defaults: nothing is listening.
    let auth = Authenticator::production();
    assert!(
        !auth.is_enabled(),
        "with default configuration the diagnostics endpoint is not enabled"
    );

    // When explicitly enabled, the bound address is loopback — never 0.0.0.0.
    let listener = SocketListener::bind_loopback().expect("an explicitly enabled endpoint binds");
    let addr = listener
        .local_addr()
        .expect("the listener reports its address")
        .expect("a TCP address");
    assert!(
        addr.ip().is_loopback(),
        "the endpoint binds loopback, not {}",
        addr.ip()
    );
}

#[doc = include_str!("spec_ch9_devtools.doc/a_read_only_client_cannot_inject_events_or_issue_control.md")]
#[test]
fn a_read_only_client_cannot_inject_events_or_issue_control() {
    let mut auth = Authenticator::new();
    auth.register_token("read-only", ScopeSet::single(PermissionScope::Read));
    let mut endpoint = DevToolsEndpoint::new(
        auth,
        obscur_kit::devtools_server::bridge::RuntimeBridge::new(
            obscur_kit::devtools_server::bridge::NullInspectSource,
        ),
    );
    endpoint.enable();
    assert!(
        endpoint
            .handshake(&HandshakeRequest::new(client_versions(), "read-only"))
            .is_accepted(),
        "the read-only client authenticates"
    );

    // (a) event injection is refused.
    let inject = endpoint.dispatch(&CommandMessage {
        id: CommandId::new(1),
        kind: CommandKind::Inject(InjectedEvent::PointerDown { x: 10, y: 10 }),
    });
    assert!(
        matches!(
            inject,
            Err(ProtocolError::Unauthorized {
                required: PermissionScope::Inject
            })
        ),
        "a read-only client is refused injection, got {inject:?}"
    );

    // (b) control is refused.
    let control = endpoint.dispatch(&CommandMessage {
        id: CommandId::new(2),
        kind: CommandKind::Control(ControlAction::StepFrame),
    });
    assert!(
        matches!(
            control,
            Err(ProtocolError::Unauthorized {
                required: PermissionScope::Control
            })
        ),
        "a read-only client is refused control, got {control:?}"
    );

    // (c) a read request in the same session still succeeds.
    let read = endpoint.dispatch(&CommandMessage {
        id: CommandId::new(3),
        kind: CommandKind::Inspect(Inspectable::Tree),
    });
    assert!(read.is_ok(), "the read permission still works: {read:?}");

    // And nothing was injected alongside the errors.
    assert!(
        endpoint.drain_controls().is_empty(),
        "the refused control action did not also take effect"
    );
}

#[doc = include_str!("spec_ch9_devtools.doc/dangerous_operations_are_audited_and_test_ids_never_affect_rendering.md")]
#[test]
fn dangerous_operations_are_audited_and_test_ids_never_affect_rendering() {
    let mut auth = Authenticator::new();
    auth.register_token("full", ScopeSet::all());
    let mut endpoint = DevToolsEndpoint::new(
        auth,
        obscur_kit::devtools_server::bridge::RuntimeBridge::new(
            obscur_kit::devtools_server::bridge::NullInspectSource,
        ),
    );
    endpoint.enable();
    assert!(
        endpoint
            .handshake(&HandshakeRequest::new(client_versions(), "full"))
            .is_accepted(),
        "the control-permitted client authenticates"
    );
    let _ = endpoint.dispatch(&CommandMessage {
        id: CommandId::new(1),
        kind: CommandKind::Inject(InjectedEvent::PointerDown { x: 5, y: 5 }),
    });
    assert!(
        !endpoint.audit_log().is_empty(),
        "the injection produced an audit record"
    );

    // `test-id` must be inert: identical geometry and no selector match.
    let css = ".a{width:100px;height:50px}[test-id]{width:999px}";
    let plain = drive(VW, VW, css, &view(vec![el(1, "a", vec![])]));
    let tagged = drive(
        VW,
        VW,
        css,
        &view(vec![el(1, "a", vec![]).with_prop(
            obscur_kit::view::view::ViewProperty::new(
                obscur_kit::identity::InternedSymbol::from_raw(
                    obscur_kit::ui_ir::name_binding::attr_symbol("test-id").raw(),
                ),
                obscur_kit::view::view::ViewValue::Text("x".into()),
            ),
        )]),
    );
    let (px, py, pw, ph) = geom(&plain, at(&plain, &[0]));
    let (tx, ty, tw, th) = geom(&tagged, at(&tagged, &[0]));
    approx(tx, px, "test-id does not move the node");
    approx(ty, py, "test-id does not move the node");
    approx(tw, pw, "no selector matches on test-id");
    approx(th, ph, "no selector matches on test-id");
    let _ = compile(css);
}
