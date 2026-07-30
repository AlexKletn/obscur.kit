//! §2.6's routing chain, reached from an application: links 1 and 3, by name.
//!
//! `UiRuntime` implements the whole chain — *focused node → focus scope →
//! ancestors → window → application fallback* — but the four positional links are
//! keyed by a `NodeId`, and a `NodeId` does not exist until a frame has built the
//! tree. An application configuring itself before the loop therefore had exactly
//! one link available (the application fallback), and the observable costs were one
//! flat command namespace, a symbol that could not mean two things in two places,
//! and handlers that could not sit beside the markup they serve.
//!
//! What closes it is not a new routing mechanism but an *identifier that exists
//! before the tree does*: §7.5's stable `test-id` / `automation-id`, written on the
//! markup itself and resolved against the live tree after every commit. So these
//! cases go the whole way an application does — author a `test-id`, name it in a
//! registration, click the element, watch the right handler run — using only
//! `obscur_kit`'s public API.
//!
//! The host contract is the pair of calls in [`Host::frame`]: run the frame, then
//! `AppControl::apply`. That is exactly what `App`'s window runner does, and it is
//! why these cases can prove the standalone entry point's behaviour without a
//! display (the windowed `App` needs `window-winit` + `render-vello`, so a test that
//! constructed one would be silently skipped in the default feature set).
#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use obscur_kit::bootstrap::CssPipeline;
use obscur_kit::control::AppControl;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::headless::input::SelectorTarget;
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::event::{EventTrust, ListenerPhase, UiEvent};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::screen::OkuiScreen;
use obscur_kit::view::view::{
    CLASS, EVENT_CLICK, TEST_ID, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty,
    ViewValue,
};

/// The one command symbol both rows bind — the same in the markup (`on:click=pick`)
/// and in Rust, because the front end hashes a handler name with the same FNV-1a
/// [`InternedSymbol::named`] uses.
const PICK: InternedSymbol = InternedSymbol::named("pick");

/// Two rows, each 100x50, stacked in a 100x100 panel: row `a` covers y ∈ [0, 50),
/// row `b` covers y ∈ [50, 100).
const CSS: &str = ".panel { width: 100px; height: 100px } \
                  .row { width: 100px; height: 50px }";

/// Markup with the stable identifiers **authored on the elements**, which is the
/// point: the name a registration uses is written where the element is.
const SCREEN: &str = r#"<div class="panel">
    <div class="row" test-id="a" on:click=pick/>
    <div class="row" test-id="b" on:click=pick/>
</div>"#;

/// The two calls a host makes per frame, and nothing else — the shape `App`'s
/// window runner has around its winit event loop.
struct Host {
    runtime: UiRuntime,
    css: CssPipeline,
    control: AppControl,
}

impl Host {
    /// A runtime with `CSS` installed through the control handle (as the runner
    /// does, so §4.12's source identity is decided in one place) and `builder` as its
    /// root. No frame has run yet, so no name can have resolved.
    fn new(builder: impl FnMut() -> View + 'static) -> Self {
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
        let mut css = CssPipeline::new();
        let control = AppControl::new();
        runtime.set_root_view(builder);
        control.set_stylesheet(CSS);
        control.apply(&mut runtime, &mut css);
        Self {
            runtime,
            css,
            control,
        }
    }

    /// Runs one reactive frame and then applies what the application queued —
    /// including resolving every positional registration against the tree the frame
    /// just committed.
    fn frame(&mut self) {
        self.runtime.request_frame(FrameReason::UiCommit);
        self.runtime
            .run_due_frame_reactive()
            .expect("a requested frame is due");
        self.control.apply(&mut self.runtime, &mut self.css);
    }

    /// A trusted press and release at `(x, y)` — a real click, as a platform adapter
    /// delivers it (an untrusted event does not run a gated default action).
    fn click(&mut self, x: f32, y: f32) {
        for kind in [PointerEventKind::Down, PointerEventKind::Up] {
            self.runtime.handle_pointer(
                PointerEvent::new(
                    PointerId::MOUSE,
                    PointerDevice::Mouse,
                    kind,
                    Point::new(x, y),
                )
                .as_trusted(),
            );
        }
    }

    /// The node a `test-id` is currently bound at, as the application sees it.
    fn bound(&self, test_id: &str) -> Option<NodeId> {
        self.control.bound_node(&SelectorTarget::test_id(test_id))
    }
}

/// A host whose root is the compiled `.okui` [`SCREEN`], with one frame committed
/// and every registration bound.
fn screen_host() -> Host {
    let screen = OkuiScreen::compile(SCREEN).expect("the screen compiles");
    Host::new(move || screen.view())
}

/// **The case that was unreachable.** One command symbol, two elements, two
/// handlers — and each click runs the handler belonging to the element it landed on.
#[test]
fn a_positional_handler_runs_for_its_own_node_and_not_the_other() {
    let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
    let mut host = screen_host();
    for name in ["a", "b"] {
        let sink = Rc::clone(&log);
        host.control
            .on_command_at(SelectorTarget::test_id(name), PICK, move || {
                sink.borrow_mut().push(name);
            });
    }
    host.frame();

    assert_ne!(
        host.bound("a"),
        None,
        "a `test-id` authored in the markup resolved to a real node"
    );
    assert_ne!(host.bound("a"), host.bound("b"), "two distinct elements");

    host.click(10.0, 10.0);
    host.click(10.0, 75.0);
    host.click(10.0, 10.0);

    assert_eq!(
        *log.borrow(),
        vec!["a", "b", "a"],
        "one symbol, two positions, and the position decides which logic runs"
    );
}

/// A handler registered positionally is handed §2.3's event, so §2.4's payload and
/// the trust flag arrive with it — the same widening the window-level registration
/// got, at the link that could not be reached at all.
#[test]
fn a_positional_handler_reads_the_event_payload() {
    let seen: Rc<Cell<Option<(Option<NodeId>, Point, bool)>>> = Rc::new(Cell::new(None));
    let mut host = screen_host();
    {
        let sink = Rc::clone(&seen);
        host.control.on_command_at(
            SelectorTarget::test_id("b"),
            PICK,
            move |event: &mut UiEvent| {
                let pointer = event.pointer().expect("a click carries its pointer");
                sink.set(Some((
                    event.target(),
                    pointer.position(),
                    event.trust() == EventTrust::Trusted,
                )));
            },
        );
    }
    host.frame();

    host.click(25.0, 75.0);

    let (target, position, trusted) = seen.get().expect("the handler ran");
    assert_eq!(target, host.bound("b"), "the node that was hit");
    assert_eq!(position, Point::new(25.0, 75.0));
    assert!(trusted, "input a platform adapter vouched for");
}

/// §2.6's order, observed from an application: a handler at the node claims the
/// command, so the window-level handler for the same symbol never sees it — while a
/// click on an element with no positional handler falls through to that window link.
#[test]
fn a_node_handler_claims_the_command_before_the_window_link() {
    let at_node = Rc::new(Cell::new(0_u32));
    let at_window = Rc::new(Cell::new(0_u32));
    let mut host = screen_host();
    {
        let counter = Rc::clone(&at_node);
        host.control
            .on_command_at(SelectorTarget::test_id("a"), PICK, move || {
                counter.set(counter.get() + 1);
            });
        let counter = Rc::clone(&at_window);
        host.runtime
            .on_command(PICK, move || counter.set(counter.get() + 1));
    }
    host.frame();

    host.click(10.0, 10.0);
    assert_eq!(
        (at_node.get(), at_window.get()),
        (1, 0),
        "link 1 claimed it and the chain stopped"
    );

    host.click(10.0, 75.0);
    assert_eq!(
        (at_node.get(), at_window.get()),
        (1, 1),
        "the other row has no handler of its own, so the window link answers"
    );
}

/// §2.3's capture phase, by name: a listener on the panel sees a command aimed at a
/// row *before* the row does, and can end the propagation there — which is how an
/// overlay or a gesture arbiter takes an interaction away.
#[test]
fn a_capture_listener_registered_by_name_sees_and_can_stop_a_descendants_command() {
    let observed = Rc::new(Cell::new(None));
    let at_node = Rc::new(Cell::new(0_u32));
    let screen = OkuiScreen::compile(
        r#"<div class="panel" test-id="panel">
            <div class="row" test-id="a" on:click=pick/>
        </div>"#,
    )
    .expect("the screen compiles");
    let mut host = Host::new(move || screen.view());
    {
        let sink = Rc::clone(&observed);
        host.control.add_event_listener(
            SelectorTarget::test_id("panel"),
            PICK,
            ListenerPhase::Capture,
            move |event: &mut UiEvent| {
                // `target` is the row that was hit; `current_target` is the panel the
                // listener sits on — the pair that makes a capture listener useful.
                sink.set(Some((event.target(), event.current_target())));
                event.stop_propagation();
            },
        );
        let counter = Rc::clone(&at_node);
        host.control
            .on_command_at(SelectorTarget::test_id("a"), PICK, move || {
                counter.set(counter.get() + 1);
            });
    }
    host.frame();

    host.click(10.0, 10.0);

    assert_eq!(
        observed.get(),
        Some((host.bound("a"), host.bound("panel"))),
        "the ancestor saw the descendant's command, and knew both ends of it"
    );
    assert_eq!(
        at_node.get(),
        0,
        "a stop in the capture phase reaches the chain too, so the row's handler never ran"
    );
}

/// A name that matches no element is not an error and not a panic — it stays
/// pending, is retried after every commit, and is *visible*: `bound_node` answers
/// `None`, which is how an application finds a misspelled `test-id` instead of
/// wondering why a button does nothing.
#[test]
fn a_name_that_matches_nothing_stays_pending_and_never_fires() {
    let ran = Rc::new(Cell::new(0_u32));
    let mut host = screen_host();
    {
        let counter = Rc::clone(&ran);
        host.control
            .on_command_at(SelectorTarget::test_id("nope"), PICK, move || {
                counter.set(counter.get() + 1);
            });
    }
    host.frame();
    host.frame();

    assert_eq!(host.bound("nope"), None);
    assert_eq!(host.control.pending_bindings(), 1, "still waiting, retried");
    host.click(10.0, 10.0);
    assert_eq!(ran.get(), 0);
}

/// A registration follows the **name**, not the node id: when the element stops
/// carrying the `test-id` the handler is released, and when it comes back the name
/// resolves again and the same handler runs. Without that, a positional registration
/// would be a one-frame accident on any UI that rebuilds a subtree — or, worse, would
/// keep answering for an element that gave the name up.
///
/// Note what the reconciler actually does here: the row keeps its component identity,
/// so removing its `test-id` **patches the node in place** rather than replacing it.
/// The node is therefore still live, and a binding that only checked liveness would
/// have stayed attached to an element that no longer answers to the name.
#[test]
fn a_registration_follows_the_name_across_a_rebuild() {
    // A Rust view builder, because the shape has to change between frames and the
    // run-time `.okui` route materialises structure statically.
    let present = Rc::new(Cell::new(true));
    let shown = Rc::clone(&present);
    let mut host = Host::new(move || {
        let mut rows = vec![row("a")];
        if !shown.get() {
            rows = vec![untagged_row()];
        }
        View::single(
            ViewNode::element(ComponentId::from_raw(0x9010))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
                .with_children(ViewFragment::from_nodes(rows)),
        )
    });
    let ran = Rc::new(Cell::new(0_u32));
    {
        let counter = Rc::clone(&ran);
        host.control
            .on_command_at(SelectorTarget::test_id("a"), PICK, move || {
                counter.set(counter.get() + 1);
            });
    }
    host.frame();
    let first = host.bound("a").expect("bound after the first commit");
    host.click(10.0, 10.0);
    assert_eq!(ran.get(), 1);

    // Take the identifier away: the binding is released rather than left pointing at
    // an element that no longer answers to the name.
    present.set(false);
    host.frame();
    assert_eq!(
        host.bound("a"),
        None,
        "nothing carries the name any more, live node or not"
    );
    host.click(10.0, 10.0);
    assert_eq!(ran.get(), 1, "and the released handler does not run");

    // Bring it back: the name resolves again, and the handler — the same closure,
    // never consumed by a registration — runs once more.
    present.set(true);
    host.frame();
    let second = host.bound("a").expect("re-bound after the rebuild");
    assert_eq!(first, second, "the reconciler patched the row in place");
    host.click(10.0, 10.0);
    assert_eq!(ran.get(), 2);
}

/// A row tagged with a stable `test-id` and bound to [`PICK`] — the Rust equivalent
/// of the markup in [`SCREEN`].
fn row(test_id: &str) -> ViewNode {
    untagged_row().with_prop(ViewProperty::new(TEST_ID, ViewValue::Text(test_id.into())))
}

/// The same row without any stable identifier.
fn untagged_row() -> ViewNode {
    ViewNode::element(ComponentId::from_raw(0x9011))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
        .with_event(ViewEventBinding::new(EVENT_CLICK, PICK))
}
