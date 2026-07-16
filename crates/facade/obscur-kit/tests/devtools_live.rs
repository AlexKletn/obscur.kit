//! End-to-end: an external DevTools client reads a **live** runtime's real tree
//! over the socket (spec §7.6). A headless `UiRuntime` renders a scene, the
//! facade bridges it into a `SnapshotInspectSource`, a `DevToolsEndpoint` serves
//! it over a real loopback socket, and a `SocketClient` handshakes and inspects
//! — proving the observe half of the agent loop works for an external process,
//! not just in-process.
//!
//! The lower half covers the **pushed** direction (§7-C): a subscriber over the
//! same real socket receives events a live runtime genuinely produced, and a host
//! that cannot push denies the subscription instead of granting it.
#![forbid(unsafe_code)]

use std::thread;
use std::time::Duration;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::devtools::{DiagnosticEventSink, RuntimeEventPublisher, inspect_source_from};
use obscur_kit::devtools_protocol::message::{
    CommandId, CommandKind, CommandMessage, EventKind, EventSeverity, HandshakeRequest,
    Inspectable, ProtocolEvent, ProtocolVersion, ScopeSet, Subscription, Topic, VersionRange,
};
use obscur_kit::devtools_server::auth::Authenticator;
use obscur_kit::devtools_server::bridge::{NullInspectSource, RuntimeBridge};
use obscur_kit::devtools_server::endpoint::DevToolsEndpoint;
use obscur_kit::devtools_server::transport::{
    ClientBound, EventQueue, InjectionQueue, SocketClient, SocketListener, SocketTransport,
};
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, InternedSymbol, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::component::COMPONENT_PROPS_FAILED;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::component::{
    Component, ComponentContext, ComponentContract, ComponentFactory, ComponentInstance,
    ComponentRegistration, ComponentVersion, ErasedComponent, FromProps, InstanceId, PropsError,
    decode_required,
};
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A component whose `step` prop decodes only from an integer — so a text value
/// fails its render and drives the runtime's real §1.6 failure path.
const GUARDED: ComponentId = ComponentId::from_raw(0x00C0_FFEE);

/// A non-reserved prop symbol, resolved the way the view tree resolves it.
fn step() -> InternedSymbol {
    InternedSymbol::from_raw(0x8000_0021)
}

struct StepProps {
    #[expect(dead_code, reason = "the decode is the point; the value is not read")]
    step: u32,
}

impl FromProps for StepProps {
    fn from_props(props: &[ViewProperty]) -> Result<Self, PropsError> {
        Ok(Self {
            step: decode_required(props, step())?,
        })
    }
}

struct Guarded;

impl Component for Guarded {
    type Props = StepProps;
    fn render(&mut self, _ctx: &mut ComponentContext<'_>, _props: &Self::Props) -> View {
        View::from_fragment(ViewFragment::from_nodes(vec![ViewNode::text("healthy")]))
    }
}

#[derive(Debug)]
struct GuardedFactory;

impl ComponentFactory for GuardedFactory {
    fn component_id(&self) -> ComponentId {
        GUARDED
    }
    fn contract(&self) -> ComponentContract {
        ComponentContract::new(GUARDED)
    }
    fn create_instance(&self, instance: InstanceId) -> ComponentInstance {
        ComponentInstance::new(instance, GUARDED)
    }
    fn create_component(&self) -> Box<dyn ErasedComponent> {
        Box::new(Guarded)
    }
}

/// A `<Guarded step=...>` element; `step` decodes only when it is an int.
fn guarded_view(step_value: ViewValue) -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(GUARDED).with_prop(ViewProperty::new(step(), step_value)),
    ]))
}

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

    // Serve one session with inject scope on its own thread — which is the real
    // shape of a host: the endpoint owns the session (token, scopes, audit log)
    // and a `UiRuntime` is not `Send`, so the runtime stays here. The host holds
    // the `InjectionQueue` and hands the endpoint only the push end, so what the
    // client injects lands on a queue THIS thread can drain.
    let injections = InjectionQueue::new();
    let sink = injections.sink();
    let listener = SocketListener::bind_loopback().expect("bind");
    let addr = listener.local_addr().expect("addr").expect("tcp addr");
    let handle = thread::spawn(move || {
        let mut auth = Authenticator::new();
        auth.register_token("agent", ScopeSet::all());
        let mut endpoint = DevToolsEndpoint::new(auth, RuntimeBridge::new(NullInspectSource))
            .with_injection_sink(sink);
        endpoint.enable();
        let stream = listener.accept().expect("accept");
        SocketTransport::new(stream)
            .serve(&mut endpoint)
            .expect("serve");
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
        let reply = client
            .command(&CommandMessage::new(
                CommandId::new(id as u64 + 1),
                CommandKind::Inject(event),
            ))
            .expect("inject io");
        // The acknowledgement is the endpoint's claim that the event was
        // delivered — assert it, because the whole §7-C finding was that this
        // very reply was a lie.
        assert_eq!(
            reply,
            ClientBound::Acknowledged(CommandId::new(id as u64 + 1)),
            "a host with a runtime acknowledges"
        );
    }
    drop(client);
    handle.join().expect("join");

    // Host drains the queued injections — on its own thread, from its own handle
    // — and applies them to the runtime.
    let drained = injections.drain();
    assert_eq!(
        drained.len(),
        2,
        "both injected events reached the host's queue"
    );
    for event in drained {
        apply_injected(&mut runtime, event);
    }

    assert!(
        clicked.get(),
        "the externally injected click dispatched the button command"
    );
}

/// The disjoint half of the test above: an inspect-only host — one that serves
/// real live state but installs no injection sink — refuses the click instead of
/// acknowledging it (§7-C).
#[test]
fn an_inspect_only_host_refuses_an_injected_click_over_the_socket() {
    use obscur_kit::devtools_protocol::error::ProtocolError;
    use obscur_kit::devtools_protocol::message::InjectedEvent;

    let sheet = compile(".btn { width: 100px; height: 40px }");
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    runtime.render_now(&View::default(), std::slice::from_ref(&sheet));

    // A host that publishes its real tree, and nothing else: no runtime is
    // reachable from the serving thread, which is precisely the shape every
    // windowed host in this workspace has.
    let source = inspect_source_from(&runtime);
    let listener = SocketListener::bind_loopback().expect("bind");
    let addr = listener.local_addr().expect("addr").expect("tcp addr");
    let handle = thread::spawn(move || {
        let mut auth = Authenticator::new();
        auth.register_token("agent", ScopeSet::all());
        let mut endpoint = DevToolsEndpoint::new(auth, RuntimeBridge::new(source));
        endpoint.enable();
        let stream = listener.accept().expect("accept");
        SocketTransport::new(stream)
            .serve(&mut endpoint)
            .expect("serve");
    });

    let mut client = SocketClient::connect_loopback(addr).expect("connect");
    let request = HandshakeRequest::new(VersionRange::exact(ProtocolVersion::CURRENT), "agent");
    let response = client.handshake(&request).expect("handshake");
    assert!(response.is_accepted(), "the inject scope IS granted here");

    let reply = client
        .command(&CommandMessage::new(
            CommandId::new(1),
            CommandKind::Inject(InjectedEvent::PointerDown { x: 50, y: 20 }),
        ))
        .expect("inject io");
    assert_eq!(
        reply,
        ClientBound::Error(ProtocolError::InjectionUnavailable),
        "a host with nowhere to deliver a click must not acknowledge one"
    );

    // Reading the same session still works — the refusal is about the runtime's
    // reachability, not about the session.
    let reply = client
        .command(&CommandMessage::new(
            CommandId::new(2),
            CommandKind::Inspect(Inspectable::Tree),
        ))
        .expect("inspect io");
    assert!(matches!(reply, ClientBound::Snapshot(_)), "{reply:?}");

    drop(client);
    handle.join().expect("join");
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

// ---- the pushed half: a subscription that delivers (§7-C) --------------------

/// Binds a loopback host that serves ONE session with `scopes`, pushing whatever
/// the host thread puts on `events`. Returns its address and the server thread.
fn spawn_pushing_host(
    events: EventQueue,
    scopes: ScopeSet,
) -> (std::net::SocketAddr, thread::JoinHandle<()>) {
    let listener = SocketListener::bind_loopback().expect("bind");
    let addr = listener.local_addr().expect("addr").expect("tcp addr");
    let handle = thread::spawn(move || {
        let mut auth = Authenticator::new();
        auth.register_token("dev", scopes);
        let mut endpoint = DevToolsEndpoint::new(auth, RuntimeBridge::new(NullInspectSource));
        endpoint.enable();
        let stream = listener.accept().expect("accept");
        SocketTransport::new(stream)
            .serve_with_events(&mut endpoint, &events, Duration::from_millis(5))
            .expect("serve");
    });
    (addr, handle)
}

fn dev_handshake() -> HandshakeRequest {
    HandshakeRequest::new(VersionRange::exact(ProtocolVersion::CURRENT), "dev")
}

/// Subscribes `client` to `topic` and returns the granted record.
fn subscribe(client: &mut SocketClient, id: u64, topic: Topic) -> Subscription {
    let reply = client
        .command(&CommandMessage::new(
            CommandId::new(id),
            CommandKind::Subscribe(topic),
        ))
        .expect("subscribe io");
    let ClientBound::Subscription(subscription) = reply else {
        panic!("expected a subscription record, got {reply:?}");
    };
    subscription
}

/// Blocks for one pushed event, failing rather than hanging if none arrives.
fn next_event(client: &mut SocketClient) -> ProtocolEvent {
    client
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout");
    let received = client
        .receive()
        .expect("io")
        .expect("an event was pushed before the timeout");
    let ClientBound::Event(event) = received else {
        panic!("expected a pushed event, got {received:?}");
    };
    event
}

#[test]
fn a_subscriber_is_pushed_the_live_runtimes_committed_frame() {
    // THE §7-C gap: `Subscribe` was answered `granted: true` and then dropped —
    // nothing in the workspace ever built an `EventKind`, so the client waited
    // forever. Here a real runtime commits a real frame and the subscriber hears
    // about it over a real socket, with no request paired to it.
    let events = EventQueue::new();
    let (addr, handle) = spawn_pushing_host(events.clone(), ScopeSet::observer());

    let sheet = compile(".row { display: flex; width: 520px; height: 200px }");
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(520.0, 200.0));
    // Attached BEFORE the frame, so the baseline is "nothing committed yet" and
    // the event below reports a commit that genuinely happened afterwards.
    let mut publisher = RuntimeEventPublisher::attach(&runtime, events);

    let mut client = SocketClient::connect_loopback(addr).expect("connect");
    assert!(
        client
            .handshake(&dev_handshake())
            .expect("io")
            .is_accepted()
    );
    let subscription = subscribe(&mut client, 1, Topic::Frames);
    assert!(
        subscription.granted,
        "a host that pushes grants the subscription"
    );

    // The host commits a frame and states what happened, from its own thread.
    runtime.render_now(&view(), std::slice::from_ref(&sheet));
    publisher.publish(&runtime);

    let event = next_event(&mut client);
    assert_eq!(event.topic, Topic::Frames);
    assert_eq!(event.subscription, subscription.id);
    assert_eq!(
        event.kind,
        EventKind::FrameCommitted {
            id: runtime.commit_version().raw()
        },
        "the id is the runtime's real commit version, not a counter of our own"
    );

    drop(client);
    handle.join().expect("join");
}

#[test]
fn a_subscriber_is_pushed_a_real_component_failure_diagnostic() {
    // The §7.8 → §7.6 join: the runtime's one real diagnostic producer
    // (a component whose props fail to decode → `report_failure`) reaches an
    // external subscriber as a pushed protocol event.
    let events = EventQueue::new();
    let (addr, handle) = spawn_pushing_host(events.clone(), ScopeSet::observer());

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 100.0));
    runtime
        .register_component(ComponentRegistration::new(
            Box::new(GuardedFactory),
            ComponentVersion::INITIAL,
        ))
        .expect("first registration");
    runtime.set_diagnostic_sink(Box::new(DiagnosticEventSink::new(events)));

    let mut client = SocketClient::connect_loopback(addr).expect("connect");
    assert!(
        client
            .handshake(&dev_handshake())
            .expect("io")
            .is_accepted()
    );
    let subscription = subscribe(&mut client, 1, Topic::Diagnostics);
    assert!(subscription.granted);

    // `step` decodes only from an int, so a text value fails the render — the
    // real §1.6 failure path, not a synthesised diagnostic.
    runtime.render_now(&guarded_view(ViewValue::Text("2".into())), &[]);

    let event = next_event(&mut client);
    assert_eq!(event.topic, Topic::Diagnostics);
    assert_eq!(event.subscription, subscription.id);
    match event.kind {
        EventKind::Diagnostic {
            severity,
            code,
            message,
        } => {
            assert_eq!(severity, EventSeverity::Error);
            assert_eq!(&*code, COMPONENT_PROPS_FAILED.as_str());
            assert!(!message.is_empty(), "the failure explains itself");
        }
        other => panic!("expected a diagnostic event, got {other:?}"),
    }

    drop(client);
    handle.join().expect("join");
}

#[test]
fn a_subscriber_hears_nothing_from_a_topic_no_producer_feeds() {
    // Honest about what is NOT wired: `Lifecycle` has no producer, so a client
    // that subscribes to it is granted (the host does push) and then hears
    // nothing. Asserted so the gap is a recorded fact rather than a surprise.
    let events = EventQueue::new();
    let (addr, handle) = spawn_pushing_host(events.clone(), ScopeSet::observer());

    let sheet = compile(".row { width: 520px; height: 200px }");
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(520.0, 200.0));
    let mut publisher = RuntimeEventPublisher::attach(&runtime, events);

    let mut client = SocketClient::connect_loopback(addr).expect("connect");
    assert!(
        client
            .handshake(&dev_handshake())
            .expect("io")
            .is_accepted()
    );
    assert!(subscribe(&mut client, 1, Topic::Lifecycle).granted);

    runtime.render_now(&view(), std::slice::from_ref(&sheet));
    publisher.publish(&runtime);

    client
        .set_read_timeout(Some(Duration::from_millis(150)))
        .expect("timeout");
    assert!(
        client.receive().is_err(),
        "no producer feeds Lifecycle, so the read times out"
    );

    drop(client);
    handle.join().expect("join");
}

#[test]
fn a_request_reply_host_denies_the_subscription_it_cannot_service() {
    // The other half of the fix: a host serving with `serve` has no loop that
    // would ever push, so it must refuse rather than grant and go quiet.
    let listener = SocketListener::bind_loopback().expect("bind");
    let addr = listener.local_addr().expect("addr").expect("tcp addr");
    let handle = thread::spawn(move || {
        let mut auth = Authenticator::new();
        auth.register_token("dev", ScopeSet::observer());
        let mut endpoint = DevToolsEndpoint::new(auth, RuntimeBridge::new(NullInspectSource));
        endpoint.enable();
        let stream = listener.accept().expect("accept");
        SocketTransport::new(stream)
            .serve(&mut endpoint)
            .expect("serve");
    });

    let mut client = SocketClient::connect_loopback(addr).expect("connect");
    assert!(
        client
            .handshake(&dev_handshake())
            .expect("io")
            .is_accepted()
    );

    // The scope IS granted — this is not a permission refusal, it is the server
    // declining to promise delivery it cannot make.
    let subscription = subscribe(&mut client, 1, Topic::Frames);
    assert!(
        !subscription.granted,
        "a request/reply host must not claim a subscription it will never service"
    );

    drop(client);
    handle.join().expect("join");
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
