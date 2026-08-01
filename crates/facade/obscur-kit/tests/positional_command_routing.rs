#![doc = include_str!("positional_command_routing.doc/_module.md")]
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
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::{
    CLASS, EVENT_CLICK, TEST_ID, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty,
    ViewValue,
};

#[doc = include_str!("positional_command_routing.doc/PICK.md")]
const PICK: InternedSymbol = InternedSymbol::named("pick");

#[doc = include_str!("positional_command_routing.doc/CSS.md")]
const CSS: &str = ".panel { width: 100px; height: 100px } \
                  .row { width: 100px; height: 50px }";

#[doc = include_str!("positional_command_routing.doc/SCREEN.md")]
const SCREEN: &str = r#"<div class="panel">
    <div class="row" test-id="a" on:click=pick/>
    <div class="row" test-id="b" on:click=pick/>
</div>"#;

#[doc = include_str!("positional_command_routing.doc/Host.md")]
struct Host {
    runtime: UiRuntime,
    css: CssPipeline,
    control: AppControl,
}

impl Host {
    #[doc = include_str!("positional_command_routing.doc/new.md")]
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

    #[doc = include_str!("positional_command_routing.doc/frame.md")]
    fn frame(&mut self) {
        self.runtime.request_frame(FrameReason::UiCommit);
        self.runtime
            .run_due_frame_reactive()
            .expect("a requested frame is due");
        self.control.apply(&mut self.runtime, &mut self.css);
    }

    #[doc = include_str!("positional_command_routing.doc/click.md")]
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

    #[doc = include_str!("positional_command_routing.doc/bound.md")]
    fn bound(&self, test_id: &str) -> Option<NodeId> {
        self.control.bound_node(&SelectorTarget::test_id(test_id))
    }
}

#[doc = include_str!("positional_command_routing.doc/screen_host.md")]
fn screen_host() -> Host {
    let screen = OkuiScreen::compile(SCREEN).expect("the screen compiles");
    Host::new(move || screen.view())
}

#[doc = include_str!("positional_command_routing.doc/macro_host.md")]
fn macro_host() -> Host {
    Host::new(|| {
        okui! {
            <div class="panel">
                <div class="row" test-id="a" on:click={pick}/>
                <div class="row" test-id="b" on:click={pick}/>
            </div>
        }
    })
}

#[doc = include_str!("positional_command_routing.doc/a_positional_handler_runs_for_its_own_node_and_not_the_other.md")]
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

#[doc = include_str!("positional_command_routing.doc/a_test_id_authored_in_the_macro_routes_to_its_own_node.md")]
#[test]
fn a_test_id_authored_in_the_macro_routes_to_its_own_node() {
    let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
    let mut host = macro_host();
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
        "the macro's `test-id` reached the reserved TEST_ID symbol the resolver reads"
    );
    assert_ne!(host.bound("a"), host.bound("b"), "two distinct elements");

    host.click(10.0, 75.0);
    host.click(10.0, 10.0);

    assert_eq!(
        *log.borrow(),
        vec!["b", "a"],
        "each click ran the handler belonging to the row it landed on"
    );
}

#[doc = include_str!("positional_command_routing.doc/a_positional_handler_reads_the_event_payload.md")]
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

#[doc = include_str!("positional_command_routing.doc/a_node_handler_claims_the_command_before_the_window_link.md")]
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

#[doc = include_str!("positional_command_routing.doc/a_capture_listener_registered_by_name_sees_and_can_stop_a_descendants_command.md")]
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

#[doc = include_str!("positional_command_routing.doc/a_name_that_matches_nothing_stays_pending_and_never_fires.md")]
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

#[doc = include_str!("positional_command_routing.doc/a_registration_follows_the_name_across_a_rebuild.md")]
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

#[doc = include_str!("positional_command_routing.doc/row.md")]
fn row(test_id: &str) -> ViewNode {
    untagged_row().with_prop(ViewProperty::new(TEST_ID, ViewValue::Text(test_id.into())))
}

#[doc = include_str!("positional_command_routing.doc/untagged_row.md")]
fn untagged_row() -> ViewNode {
    ViewNode::element(ComponentId::from_raw(0x9011))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
        .with_event(ViewEventBinding::new(EVENT_CLICK, PICK))
}
