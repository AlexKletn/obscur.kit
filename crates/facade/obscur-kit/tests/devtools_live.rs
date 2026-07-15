//! End-to-end: an external DevTools client reads a **live** runtime's real tree
//! over the socket (spec §7.6). A headless `UiRuntime` renders a scene, the
//! facade bridges it into a `SnapshotInspectSource`, a `DevToolsEndpoint` serves
//! it over a real loopback socket, and a `SocketClient` handshakes and inspects
//! — proving the observe half of the agent loop works for an external process,
//! not just in-process.
#![forbid(unsafe_code)]

use std::thread;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::devtools::inspect_source_from;
use obscur_kit::devtools_protocol::message::{
    CommandId, CommandKind, CommandMessage, HandshakeRequest, Inspectable, ProtocolVersion,
    ScopeSet, VersionRange,
};
use obscur_kit::devtools_server::auth::Authenticator;
use obscur_kit::devtools_server::bridge::RuntimeBridge;
use obscur_kit::devtools_server::endpoint::DevToolsEndpoint;
use obscur_kit::devtools_server::transport::{
    ClientBound, SocketClient, SocketListener, SocketTransport,
};
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// A `.row` flex container with three cards `.a`, `.b`, `.c`.
fn view() -> View {
    let container = ComponentId::from_raw(0x00C0_0001);
    let card = |class: &str| {
        ViewNode::element(container).with_prop(ViewProperty::new(
            CLASS,
            ViewValue::Text(format!("card {class}").into()),
        ))
    };
    let row = ViewNode::element(container)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
        .with_children(ViewFragment::from_nodes(vec![
            card("a"),
            card("b"),
            card("c"),
        ]));
    View::from_fragment(ViewFragment::from_nodes(vec![row]))
}

#[test]
fn external_client_inspects_the_live_tree_over_the_socket() {
    // 1. Render a scene headlessly so the runtime has a real tree + boxes.
    let sheet = compile(
        ".row { display: flex; flex-direction: row; gap: 20px; padding: 20px; \
                width: 520px; height: 200px }\n\
         .card { width: 140px; height: 160px }\n",
    );
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(520.0, 200.0));
    runtime.render_now(&view(), std::slice::from_ref(&sheet));

    // 2. Bridge the live runtime into an inspect source and serve it.
    let source = inspect_source_from(&runtime);
    let listener = SocketListener::bind_loopback().expect("bind");
    let addr = listener.local_addr().expect("addr").expect("tcp addr");
    let handle = thread::spawn(move || {
        let mut auth = Authenticator::new();
        auth.register_token("dev", ScopeSet::all_read());
        let mut endpoint = DevToolsEndpoint::new(auth, RuntimeBridge::new(source));
        endpoint.enable();
        let stream = listener.accept().expect("accept");
        SocketTransport::new(stream)
            .serve(&mut endpoint)
            .expect("serve");
    });

    // 3. External client: handshake, then inspect the tree.
    let mut client = SocketClient::connect_loopback(addr).expect("connect");
    let request = HandshakeRequest::new(VersionRange::exact(ProtocolVersion::CURRENT), "dev");
    assert!(client.handshake(&request).expect("handshake").is_accepted());

    let reply = client
        .command(&CommandMessage::new(
            CommandId::new(1),
            CommandKind::Inspect(Inspectable::Tree),
        ))
        .expect("inspect io");
    let ClientBound::Snapshot(snapshot) = reply else {
        panic!("expected a tree snapshot, got {reply:?}");
    };

    // The payload carries the REAL live tree: node count > 0 and the detail
    // names the middle card at its resolved absolute geometry.
    assert!(snapshot.payload.entity_count >= 4, "root + 3 cards");
    assert!(
        snapshot.payload.detail.contains(".card.b"),
        "{}",
        snapshot.payload.detail
    );
    assert!(
        snapshot.payload.detail.contains("180,20"),
        "{}",
        snapshot.payload.detail
    );

    // Layout target carries just the geometry lines.
    let layout = client
        .command(&CommandMessage::new(
            CommandId::new(2),
            CommandKind::Inspect(Inspectable::Layout),
        ))
        .expect("inspect io");
    let ClientBound::Snapshot(layout) = layout else {
        panic!("expected a layout snapshot");
    };
    assert!(
        layout.payload.detail.contains("140×160"),
        "{}",
        layout.payload.detail
    );

    drop(client);
    handle.join().expect("join");
}

#[test]
fn external_client_injects_a_click_that_drives_the_runtime() {
    use std::cell::Cell;
    use std::rc::Rc;

    use obscur_kit::devtools::apply_injected;
    use obscur_kit::devtools_protocol::message::InjectedEvent;
    use obscur_kit::devtools_server::bridge::NullInspectSource;
    use obscur_kit::identity::InternedSymbol;
    use obscur_kit::view::view::{EVENT_CLICK, ViewEventBinding};

    const CMD: InternedSymbol = InternedSymbol::from_raw(0x0ABC);

    // A `.btn` element (100×40 at the origin) bound to fire CMD on click, with a
    // handler that flips a shared flag.
    let sheet = compile(".btn { width: 100px; height: 40px }");
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    let clicked = Rc::new(Cell::new(false));
    let flag = Rc::clone(&clicked);
    runtime.on_command(CMD, move || flag.set(true));
    let btn = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(0x00C0_0001))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("btn".into())))
            .with_event(ViewEventBinding::new(EVENT_CLICK, CMD)),
    ]));
    runtime.render_now(&btn, std::slice::from_ref(&sheet));

    // Serve one session with inject scope; the thread returns the endpoint so the
    // host (this thread) can drain what the client injected.
    let listener = SocketListener::bind_loopback().expect("bind");
    let addr = listener.local_addr().expect("addr").expect("tcp addr");
    let handle = thread::spawn(move || {
        let mut auth = Authenticator::new();
        auth.register_token("agent", ScopeSet::all());
        let mut endpoint = DevToolsEndpoint::new(auth, RuntimeBridge::new(NullInspectSource));
        endpoint.enable();
        let stream = listener.accept().expect("accept");
        SocketTransport::new(stream)
            .serve(&mut endpoint)
            .expect("serve");
        endpoint
    });

    // The external agent injects a press+release over the socket at the button.
    let mut client = SocketClient::connect_loopback(addr).expect("connect");
    let request = HandshakeRequest::new(VersionRange::exact(ProtocolVersion::CURRENT), "agent");
    assert!(client.handshake(&request).expect("handshake").is_accepted());
    for (id, event) in [
        InjectedEvent::PointerDown { x: 50, y: 20 },
        InjectedEvent::PointerUp { x: 50, y: 20 },
    ]
    .into_iter()
    .enumerate()
    {
        client
            .command(&CommandMessage::new(
                CommandId::new(id as u64 + 1),
                CommandKind::Inject(event),
            ))
            .expect("inject io");
    }
    drop(client);

    // Host drains the queued injections and applies them to the runtime.
    let mut endpoint = handle.join().expect("join");
    let injected = endpoint.drain_injections();
    assert_eq!(
        injected.len(),
        2,
        "both injected events reached the endpoint queue"
    );
    for event in injected {
        apply_injected(&mut runtime, event);
    }

    assert!(
        clicked.get(),
        "the externally injected click dispatched the button command"
    );
}

/// Every §7.6 inspect target the runtime can genuinely answer, from a live tree.
#[test]
fn every_target_the_runtime_knows_reports_real_state_not_a_default() {
    // The audit's §7 finding: only Tree/Layout/Styles/Focus/Rendering were
    // registered, so Animations/Resources/Caches/Accessibility/Scene all fell back
    // to typed defaults — an inspector saw "0" for state that existed.
    // The cards carry a background, so the frame genuinely paints something —
    // an unpainted tree would make the Scene assertion pass for the wrong reason.
    let sheet = compile(
        ".row { display: flex; width: 520px; height: 200px }\n\
         .card { width: 140px; height: 160px; background-color: #336699 }\n",
    );
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(520.0, 200.0));
    runtime.render_now(&view(), std::slice::from_ref(&sheet));

    let source = inspect_source_from(&runtime);
    let count = |target| {
        obscur_kit::devtools_server::bridge::InspectSource::snapshot(&source, target).entity_count
    };

    // Accessibility: the semantic tree the runtime derives — the same one a
    // platform AX bridge consumes, so an inspector and a screen reader agree.
    assert!(
        count(Inspectable::Accessibility) > 0,
        "the semantic tree is real"
    );
    // Scene: the display list this frame paints.
    assert!(
        count(Inspectable::Scene) > 0,
        "the composed paint commands are real"
    );
    // Caches / Resources / Animations are legitimately zero here — nothing was
    // loaded and nothing animates — so assert the payload DESCRIBES rather than
    // that it counts, which is what a default could never do.
    let described = |target| {
        obscur_kit::devtools_server::bridge::InspectSource::snapshot(&source, target).description
    };
    assert!(
        described(Inspectable::Resources).contains("requested"),
        "{:?}",
        described(Inspectable::Resources)
    );
    assert!(
        described(Inspectable::Caches).contains("image"),
        "{:?}",
        described(Inspectable::Caches)
    );
    assert!(
        described(Inspectable::Animations).contains("timeline"),
        "{:?}",
        described(Inspectable::Animations)
    );
}

#[test]
fn event_flow_and_plugin_state_stay_honest_defaults() {
    // Not an oversight, and not a synthesised answer: the runtime keeps no
    // dispatch log to report a flow from, and the plugin tier is a declared-
    // deferred skeleton. An inspector is told nothing rather than told a
    // confident fiction about state that does not exist.
    let sheet = compile(".row { width: 520px; height: 200px }");
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(520.0, 200.0));
    runtime.render_now(&view(), std::slice::from_ref(&sheet));

    let source = inspect_source_from(&runtime);
    for target in [Inspectable::EventFlow, Inspectable::PluginState] {
        let payload = obscur_kit::devtools_server::bridge::InspectSource::snapshot(&source, target);
        assert_eq!(payload.entity_count, 0, "{target:?} claims nothing");
    }
}
