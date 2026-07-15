//! End-to-end: input is **dispatched by the frame pipeline**, not the instant a
//! host hands it over (spec §3.4 phase 3).
//!
//! The immediate entry points (`handle_pointer` and friends) make *when* an event
//! is handled a property of when the host chose to call, and leave the ordering
//! between input, commits and frames for the host to get right. `queue_input`
//! moves that into the pipeline: events are received, the frame is chosen, and
//! only then is input dispatched — before anything commits.
//!
//! The two paths share one implementation, so the interesting assertions are not
//! "does dispatch work" but the properties only a phase can have: nothing happens
//! until the frame runs, a burst is served in order by one frame, and what
//! dispatch invalidates is folded into that same frame rather than scheduling
//! another.

#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::scheduler::BackpressureDecision;
use obscur_kit::runtime::{InputDispatch, InputEvent, UiRuntime};
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

const CSS: &str = ".page { width: 200px; height: 200px } \
                   .top { width: 100px; height: 50px; background-color: #ff0000 } \
                   .bottom { width: 100px; height: 50px; background-color: #0000ff }";

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Two stacked boxes: `.top` covers y 0..50, `.bottom` covers y 50..100.
fn view() -> View {
    let component = ComponentId::from_raw(0x00C0_0001);
    let top = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("top".into())));
    let bottom = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("bottom".into())));
    let page = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![top, bottom]));
    View::from_fragment(ViewFragment::from_nodes(vec![page]))
}

/// A rendered runtime plus the `(top, bottom)` node ids.
fn rendered() -> (UiRuntime, NodeId, NodeId) {
    let sheet = compile(CSS);
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    runtime.render_now(&view(), std::slice::from_ref(&sheet));
    let root = runtime.tree().root().expect("a root");
    let page = runtime.tree().get(root).expect("live").children()[0];
    let children = runtime.tree().get(page).expect("live").children().to_vec();
    (runtime, children[0], children[1])
}

fn moved_to(y: f32) -> InputEvent {
    InputEvent::Pointer(PointerEvent::new(
        PointerId::from_raw(1),
        PointerDevice::Mouse,
        PointerEventKind::Move,
        Point::new(10.0, y),
    ))
}

const OVER_TOP: f32 = 25.0;
const OVER_BOTTOM: f32 = 75.0;

#[test]
fn a_queued_event_is_not_dispatched_until_the_frame_runs() {
    // THE property a phase has and an immediate call cannot: queuing changes
    // nothing on its own.
    let sheet = compile(CSS);
    let (mut runtime, top, _bottom) = rendered();

    runtime.queue_input(moved_to(OVER_TOP));
    assert_eq!(runtime.pending_input(), 1, "queued, not dispatched");
    assert!(
        runtime.take_input_dispatches().is_empty(),
        "nothing has been dispatched yet"
    );

    runtime
        .run_due_frame(&view(), std::slice::from_ref(&sheet))
        .expect("queuing requested a frame");

    assert_eq!(runtime.pending_input(), 0, "the frame drained the queue");
    let dispatches = runtime.take_input_dispatches();
    assert_eq!(dispatches.len(), 1);
    let InputDispatch::Pointer(dispatch) = &dispatches[0] else {
        panic!("expected a pointer dispatch, got {dispatches:?}");
    };
    assert_eq!(dispatch.target, Some(top), "and it hit-tested the top box");
    assert!(dispatch.hover_changed);
}

#[test]
fn a_burst_is_dispatched_in_order_by_one_frame() {
    // Ordering is the pipeline's to keep, not the host's to be careful about.
    let sheet = compile(CSS);
    let (mut runtime, top, bottom) = rendered();

    runtime.queue_input(moved_to(OVER_TOP));
    runtime.queue_input(moved_to(OVER_BOTTOM));
    runtime.queue_input(moved_to(OVER_TOP));
    assert_eq!(runtime.pending_input(), 3);

    runtime
        .run_due_frame(&view(), std::slice::from_ref(&sheet))
        .expect("a frame is due");

    let targets: Vec<_> = runtime
        .take_input_dispatches()
        .iter()
        .map(|d| match d {
            InputDispatch::Pointer(p) => p.target,
            other => panic!("expected pointer dispatches, got {other:?}"),
        })
        .collect();
    assert_eq!(
        targets,
        vec![Some(top), Some(bottom), Some(top)],
        "three events, one frame, source order"
    );
}

#[test]
fn what_dispatch_invalidates_is_served_by_the_frame_that_dispatched_it() {
    // THE reason `request_frame` coalesces during the dispatch phase. A hover
    // change asks for a restyle; the frame that dispatched it has not chosen its
    // plan yet, so it serves the request itself. Left to the scheduler, every
    // burst of input would cost a second full frame that changes nothing.
    let sheet = compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, _top, _bottom) = rendered();

    // Start clean: nothing owed.
    while runtime.has_pending_frame() {
        runtime.run_due_frame(&view(), sheets);
    }
    assert!(!runtime.has_pending_frame());

    runtime.queue_input(moved_to(OVER_TOP));
    let frame = runtime
        .run_due_frame(&view(), sheets)
        .expect("a frame is due");
    assert!(
        frame.restyled(),
        "the hover the dispatch phase produced is restyled by THIS frame"
    );
    assert!(
        !runtime.has_pending_frame(),
        "and it owes no follow-up: the request the dispatch made was coalesced \
         into the frame running it"
    );
}

#[test]
fn queuing_between_frames_is_an_ordinary_admitted_request() {
    // The other side of the coalescing rule: outside the dispatch phase there is
    // no frame to absorb the request, so it goes to the scheduler like any other.
    let (mut runtime, _top, _bottom) = rendered();
    assert_eq!(
        runtime.queue_input(moved_to(OVER_TOP)),
        BackpressureDecision::Admit
    );
    assert!(runtime.has_pending_frame(), "and it really scheduled one");
}

#[test]
fn dispatches_do_not_accumulate_across_frames() {
    let sheet = compile(CSS);
    let sheets = std::slice::from_ref(&sheet);
    let (mut runtime, _top, _bottom) = rendered();

    runtime.queue_input(moved_to(OVER_TOP));
    runtime.run_due_frame(&view(), sheets);
    assert_eq!(runtime.take_input_dispatches().len(), 1);

    // A frame that dispatches nothing leaves nothing behind, so a host that never
    // reads them is not accumulating a leak.
    runtime.request_frame(obscur_kit::runtime::frame::FrameReason::Animation);
    runtime.run_due_frame(&view(), sheets);
    assert!(runtime.take_input_dispatches().is_empty());
}

#[test]
fn the_immediate_path_is_unchanged() {
    // The regression guard: `handle_pointer` still dispatches on the spot, which
    // is what every existing host does.
    let (mut runtime, top, _bottom) = rendered();
    let dispatch = runtime.handle_pointer(PointerEvent::new(
        PointerId::from_raw(1),
        PointerDevice::Mouse,
        PointerEventKind::Move,
        Point::new(10.0, OVER_TOP),
    ));
    assert_eq!(dispatch.target, Some(top));
    assert!(dispatch.hover_changed);
    assert_eq!(runtime.pending_input(), 0, "it never touches the queue");
    assert!(runtime.take_input_dispatches().is_empty());
}

#[test]
fn queued_and_immediate_dispatch_agree() {
    // One implementation, two timings: the same event must produce the same
    // dispatch either way, or the phase is a second code path pretending to be
    // the first.
    let sheet = compile(CSS);
    let sheets = std::slice::from_ref(&sheet);

    let (mut immediate, _t, _b) = rendered();
    let direct = immediate.handle_pointer(PointerEvent::new(
        PointerId::from_raw(1),
        PointerDevice::Mouse,
        PointerEventKind::Move,
        Point::new(10.0, OVER_BOTTOM),
    ));

    let (mut queued, _t, _b) = rendered();
    queued.queue_input(moved_to(OVER_BOTTOM));
    queued
        .run_due_frame(&view(), sheets)
        .expect("a frame is due");
    let dispatches = queued.take_input_dispatches();
    let InputDispatch::Pointer(through_queue) = &dispatches[0] else {
        panic!("expected a pointer dispatch");
    };

    assert_eq!(through_queue.target, direct.target);
    assert_eq!(through_queue.hover_changed, direct.hover_changed);
    assert_eq!(through_queue.focus_changed, direct.focus_changed);
    assert_eq!(through_queue.command, direct.command);
}
