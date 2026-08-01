#![doc = include_str!("event_model_delivery.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, InternedSymbol, NodeId, RuntimeId};
use obscur_kit::interaction::event::{EventPhase, EventTrust, ListenerPhase, UiEvent};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{
    CLASS, EVENT_CLICK, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

#[doc = include_str!("event_model_delivery.doc/PICK.md")]
const PICK: InternedSymbol = InternedSymbol::from_raw(0x7C01);

#[doc = include_str!("event_model_delivery.doc/PhaseLog.md")]
type PhaseLog = Rc<RefCell<Vec<(EventPhase, Option<NodeId>, Option<NodeId>)>>>;

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("event_model_delivery.doc/view.md")]
fn view() -> View {
    let row = |id: u64, class: &str| {
        ViewNode::element(ComponentId::from_raw(id))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
            .with_event(ViewEventBinding::new(EVENT_CLICK, PICK))
    };
    View::single(
        ViewNode::element(ComponentId::from_raw(0x7C10))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![
                row(0x7C11, "a"),
                row(0x7C12, "b"),
            ])),
    )
}

const CSS: &str = ".panel{width:100px;height:100px} \
                  .a{width:100px;height:50px} \
                  .b{width:100px;height:50px}";

#[doc = include_str!("event_model_delivery.doc/committed.md")]
fn committed() -> (UiRuntime, NodeId, NodeId, NodeId) {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));
    let sheet = compile(CSS);
    let _ = rt.render_now(&view(), std::slice::from_ref(&sheet));

    let root = rt.tree().root().expect("a root after one frame");
    let panel = rt
        .tree()
        .get(root)
        .expect("the root is retained")
        .child_nodes()[0];
    let rows = rt
        .tree()
        .get(panel)
        .expect("the panel is retained")
        .child_nodes()
        .to_vec();
    (rt, panel, rows[0], rows[1])
}

#[doc = include_str!("event_model_delivery.doc/click.md")]
fn click(rt: &mut UiRuntime, x: f32, y: f32) {
    for kind in [PointerEventKind::Down, PointerEventKind::Up] {
        rt.handle_pointer(
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

#[doc = include_str!("event_model_delivery.doc/a_handler_learns_which_of_two_nodes_sharing_one_symbol_was_hit.md")]
#[test]
fn a_handler_learns_which_of_two_nodes_sharing_one_symbol_was_hit() {
    let (mut rt, _panel, a, b) = committed();

    let hits: Rc<RefCell<Vec<Option<NodeId>>>> = Rc::new(RefCell::new(Vec::new()));
    {
        let sink = Rc::clone(&hits);
        rt.on_command(PICK, move |e: &mut UiEvent| {
            sink.borrow_mut().push(e.target());
        });
    }

    click(&mut rt, 10.0, 10.0);
    click(&mut rt, 10.0, 75.0);
    click(&mut rt, 10.0, 10.0);

    assert_eq!(
        *hits.borrow(),
        vec![Some(a), Some(b), Some(a)],
        "one handler, one symbol, three clicks — and each one names its own node"
    );
}

#[doc = include_str!("event_model_delivery.doc/the_pointer_payload_reaches_the_handler.md")]
#[test]
fn the_pointer_payload_reaches_the_handler() {
    let (mut rt, _panel, _a, _b) = committed();

    let seen: Rc<Cell<Option<PointerEvent>>> = Rc::new(Cell::new(None));
    let trusted = Rc::new(Cell::new(false));
    {
        let payload = Rc::clone(&seen);
        let trust = Rc::clone(&trusted);
        rt.on_command(PICK, move |e: &mut UiEvent| {
            payload.set(e.pointer());
            trust.set(e.trust() == EventTrust::Trusted);
        });
    }

    click(&mut rt, 25.0, 75.0);

    let pointer = seen.get().expect("the click carried its pointer payload");
    assert_eq!(pointer.pointer(), PointerId::MOUSE);
    assert_eq!(pointer.device(), PointerDevice::Mouse);
    assert_eq!(pointer.position(), Point::new(25.0, 75.0));
    assert_eq!(pointer.pressure(), 0.0);
    assert!(pointer.is_primary());
    assert!(pointer.hover_capable());
    assert!(
        trusted.get(),
        "input a platform adapter vouched for is trusted where the handler can see it"
    );
}

#[doc = include_str!("event_model_delivery.doc/an_application_can_observe_the_capture_target_bubble_walk.md")]
#[test]
fn an_application_can_observe_the_capture_target_bubble_walk() {
    let (mut rt, panel, a, _b) = committed();

    let log: PhaseLog = Rc::new(RefCell::new(Vec::new()));
    let record = |rt: &mut UiRuntime, node: NodeId, phase: ListenerPhase| {
        let sink = Rc::clone(&log);
        rt.add_event_listener(node, PICK, phase, move |e: &mut UiEvent| {
            sink.borrow_mut()
                .push((e.phase(), e.current_target(), e.target()));
        });
    };
    record(&mut rt, panel, ListenerPhase::Capture);
    record(&mut rt, a, ListenerPhase::Bubble);
    record(&mut rt, panel, ListenerPhase::Bubble);

    click(&mut rt, 10.0, 10.0);

    assert_eq!(
        log.borrow()
            .iter()
            .map(|(phase, current, _)| (*phase, *current))
            .collect::<Vec<_>>(),
        vec![
            (EventPhase::Capture, Some(panel)),
            (EventPhase::Target, Some(a)),
            (EventPhase::Bubble, Some(panel)),
        ]
    );
    assert!(
        log.borrow().iter().all(|(_, _, target)| *target == Some(a)),
        "`target` stays the node that was hit for the whole propagation"
    );
}

#[doc = include_str!("event_model_delivery.doc/an_application_can_steer_the_propagation.md")]
#[test]
fn an_application_can_steer_the_propagation() {
    // prevent_default: the ancestor still hears about it, the handler does not run.
    let (mut rt, panel, a, _b) = committed();
    let ancestor = Rc::new(Cell::new(0_u32));
    let handler = Rc::new(Cell::new(0_u32));
    {
        let counter = Rc::clone(&ancestor);
        rt.add_event_listener(panel, PICK, ListenerPhase::Bubble, move || {
            counter.set(counter.get() + 1);
        });
        let counter = Rc::clone(&handler);
        rt.on_command(PICK, move || counter.set(counter.get() + 1));
        rt.add_event_listener(a, PICK, ListenerPhase::Bubble, |e: &mut UiEvent| {
            e.prevent_default();
        });
    }
    click(&mut rt, 10.0, 10.0);
    assert_eq!(
        (ancestor.get(), handler.get()),
        (1, 0),
        "prevent_default cancels the default action, not the propagation"
    );

    // stop_propagation: neither runs.
    let (mut rt, panel, a, _b) = committed();
    let ancestor = Rc::new(Cell::new(0_u32));
    let handler = Rc::new(Cell::new(0_u32));
    {
        let counter = Rc::clone(&ancestor);
        rt.add_event_listener(panel, PICK, ListenerPhase::Bubble, move || {
            counter.set(counter.get() + 1);
        });
        let counter = Rc::clone(&handler);
        rt.on_command(PICK, move || counter.set(counter.get() + 1));
        rt.add_event_listener(a, PICK, ListenerPhase::Bubble, |e: &mut UiEvent| {
            e.stop_propagation();
        });
    }
    click(&mut rt, 10.0, 10.0);
    assert_eq!(
        (ancestor.get(), handler.get()),
        (0, 0),
        "a stop at the target ends the walk before the ancestor and the chain"
    );
}

#[doc = include_str!("event_model_delivery.doc/the_zero_argument_handler_shape_is_unchanged.md")]
#[test]
fn the_zero_argument_handler_shape_is_unchanged() {
    let (mut rt, panel, a, _b) = committed();

    let plain = Rc::new(Cell::new(0_u32));
    {
        let counter = Rc::clone(&plain);
        // No parameter, no annotation, no event — exactly as before this change.
        rt.on_command(PICK, move || counter.set(counter.get() + 1));
    }
    let observed = Rc::new(Cell::new(None));
    {
        let sink = Rc::clone(&observed);
        rt.add_event_listener(
            panel,
            PICK,
            ListenerPhase::Capture,
            move |e: &mut UiEvent| {
                sink.set(e.target());
            },
        );
    }

    click(&mut rt, 10.0, 10.0);

    assert_eq!(plain.get(), 1, "the zero-argument handler ran");
    assert_eq!(
        observed.get(),
        Some(a),
        "and an event-taking listener on the same symbol saw the event"
    );
}
