#![doc = include_str!("drag_and_drop.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::data_transfer::drag::{DragSource, DropTarget};
use obscur_kit::data_transfer::offer::{DataOffer, MimeType};
use obscur_kit::data_transfer::transfer::TransferPayload;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, NodeId, RuntimeId};
use obscur_kit::interaction::pointer::{PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::orchestration::DragOutcome;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

const VIEWPORT: f32 = 200.0;

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("drag_and_drop.doc/view.md")]
fn view() -> View {
    let component = ComponentId::from_raw(0x00C0_0001);
    let source = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("source".into())));
    let sink = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("sink".into())));
    let page = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![source, sink]));
    View::from_fragment(ViewFragment::from_nodes(vec![page]))
}

const CSS: &str = ".page { width: 200px; height: 200px } \
                  .source { width: 100px; height: 50px; background-color: #ff0000 } \
                  .sink { width: 100px; height: 50px; background-color: #0000ff }";

#[doc = include_str!("drag_and_drop.doc/rendered.md")]
fn rendered() -> (UiRuntime, NodeId, NodeId) {
    let sheet = compile(CSS);
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    runtime.render_now(&view(), std::slice::from_ref(&sheet));

    let root = runtime.tree().root().expect("a root");
    let page = runtime.tree().get(root).expect("live").child_nodes()[0];
    let children = runtime
        .tree()
        .get(page)
        .expect("live")
        .child_nodes()
        .to_vec();
    (runtime, children[0], children[1])
}

#[doc = include_str!("drag_and_drop.doc/at.md")]
fn at(kind: PointerEventKind, y: f32) -> PointerEvent {
    PointerEvent::new(
        PointerId::from_raw(1),
        obscur_kit::interaction::pointer::PointerDevice::Mouse,
        kind,
        Point::new(10.0, y),
    )
}

#[doc = include_str!("drag_and_drop.doc/text_drag.md")]
fn text_drag(origin: NodeId) -> DragSource {
    DragSource::new(origin, TransferPayload::new(DataOffer::text("payload")))
}

#[doc = include_str!("drag_and_drop.doc/OVER_SOURCE.md")]
const OVER_SOURCE: f32 = 25.0;
const OVER_SINK: f32 = 75.0;
const OVER_NOTHING: f32 = 150.0;

#[test]
fn a_drop_on_a_registered_target_reports_it_and_reconciles_that_node() {
    let (mut runtime, source, sink) = rendered();
    runtime.register_drop_target(DropTarget::new(sink));

    assert!(runtime.begin_drag(text_drag(source)), "the drag starts");
    runtime.handle_pointer(at(PointerEventKind::Move, OVER_SINK));
    runtime.handle_pointer(at(PointerEventKind::Up, OVER_SINK));

    let outcome = runtime.take_drag_outcome().expect("the drag finished");
    let DragOutcome::Dropped { target, .. } = outcome else {
        panic!("expected a drop on the sink, got {outcome:?}");
    };
    assert_eq!(target, sink, "the payload landed on the registered target");
    assert!(
        runtime.take_drag_outcome().is_none(),
        "the outcome is taken once, not left to be re-read"
    );
    assert!(runtime.drag_session().is_none(), "and the session is over");
}

#[doc = include_str!("drag_and_drop.doc/grown_view.md")]
fn grown_view() -> View {
    let component = ComponentId::from_raw(0x00C0_0001);
    let source = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("source".into())));
    let sink = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("sink".into())))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::element(
            ComponentId::from_raw(0x00C0_0002),
        )]));
    let page = ViewNode::element(component)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("page".into())))
        .with_children(ViewFragment::from_nodes(vec![source, sink]));
    View::from_fragment(ViewFragment::from_nodes(vec![page]))
}

#[test]
fn a_drop_commits_its_work_and_a_cancel_applies_nothing() {
    // THE commit/cancel guarantee, in observable terms.
    //
    // Both drags travel the same route over the same target and both end. The
    // dropped one commits its staged `Rerender`, so the frame it schedules is
    // classified from that operation and reconciles. The abandoned one throws the
    // identical staged work away, so the only frame it asks for is the one that
    // clears its own visuals — which reconciles nothing.
    let sheet = compile(CSS);
    let sheets = std::slice::from_ref(&sheet);

    // --- the cancelled drag: over the target, then off it, then let go.
    let (mut runtime, source, sink) = rendered();
    runtime.register_drop_target(DropTarget::new(sink));
    runtime.begin_drag(text_drag(source));
    runtime.handle_pointer(at(PointerEventKind::Move, OVER_SINK));
    // Drain everything the drag itself asked for, so what is pending after the
    // release can only be what the release caused.
    while runtime.has_pending_frame() {
        runtime.run_due_frame(&view(), sheets);
    }
    let before = runtime.tree().len();

    runtime.handle_pointer(at(PointerEventKind::Move, OVER_NOTHING));
    runtime.handle_pointer(at(PointerEventKind::Up, OVER_NOTHING));
    assert_eq!(
        runtime.take_drag_outcome().expect("it finished"),
        DragOutcome::Cancelled,
        "released over nothing"
    );
    while runtime.has_pending_frame() {
        runtime.run_due_frame(&grown_view(), sheets);
    }
    assert_eq!(
        runtime.tree().len(),
        before,
        "an abandoned drag applies nothing: its frames do not reconcile, so the \
         grown view is not adopted"
    );

    // --- the committed drag: the same route, but let go ON the target.
    let (mut runtime, source, sink) = rendered();
    runtime.register_drop_target(DropTarget::new(sink));
    runtime.begin_drag(text_drag(source));
    runtime.handle_pointer(at(PointerEventKind::Move, OVER_SINK));
    while runtime.has_pending_frame() {
        runtime.run_due_frame(&view(), sheets);
    }
    let before = runtime.tree().len();

    runtime.handle_pointer(at(PointerEventKind::Up, OVER_SINK));
    assert!(matches!(
        runtime.take_drag_outcome().expect("it finished"),
        DragOutcome::Dropped { .. }
    ));
    let frame = runtime
        .run_due_frame(&grown_view(), sheets)
        .expect("the drop's transaction scheduled a frame");
    assert!(
        frame.reconciled(),
        "the drop's `Rerender` classified a reconcile"
    );
    assert!(
        runtime.tree().len() > before,
        "so the work really happened: {} then {}",
        before,
        runtime.tree().len()
    );
}

#[test]
fn a_target_that_refuses_the_payload_is_not_dropped_on() {
    // The offer is `text/plain`; this sink takes images only.
    let (mut runtime, source, sink) = rendered();
    runtime.register_drop_target(DropTarget::new(sink).accepting(MimeType::new("image/png")));

    runtime.begin_drag(text_drag(source));
    runtime.handle_pointer(at(PointerEventKind::Move, OVER_SINK));
    runtime.handle_pointer(at(PointerEventKind::Up, OVER_SINK));

    assert_eq!(
        runtime.take_drag_outcome().expect("it finished"),
        DragOutcome::Cancelled,
        "a target that cannot take the payload is not a drop site"
    );
}

#[test]
fn an_unregistered_node_is_not_a_drop_site() {
    let (mut runtime, source, _sink) = rendered();
    // No `register_drop_target` at all.
    runtime.begin_drag(text_drag(source));
    runtime.handle_pointer(at(PointerEventKind::Move, OVER_SINK));
    runtime.handle_pointer(at(PointerEventKind::Up, OVER_SINK));

    assert_eq!(
        runtime.take_drag_outcome().expect("it finished"),
        DragOutcome::Cancelled
    );
}

#[test]
fn the_session_tracks_the_pointer_across_targets() {
    let (mut runtime, source, sink) = rendered();
    runtime.register_drop_target(DropTarget::new(sink));
    runtime.begin_drag(text_drag(source));

    runtime.handle_pointer(at(PointerEventKind::Move, OVER_SOURCE));
    let session = runtime.drag_session().expect("in flight");
    assert_eq!(session.position(), Point::new(10.0, OVER_SOURCE));
    assert_eq!(session.over(), None, "the source is not a drop target");

    runtime.handle_pointer(at(PointerEventKind::Move, OVER_SINK));
    let session = runtime.drag_session().expect("in flight");
    assert_eq!(session.position(), Point::new(10.0, OVER_SINK));
    assert_eq!(session.over(), Some(sink), "now over the sink");

    runtime.handle_pointer(at(PointerEventKind::Move, OVER_NOTHING));
    assert_eq!(
        runtime.drag_session().expect("in flight").over(),
        None,
        "and off it again"
    );
}

#[test]
fn a_release_that_ends_a_drag_does_not_also_click() {
    // A drag is not a click. If the release both dropped and activated whatever it
    // landed on, a host would see a phantom click on every drop — which is the one
    // thing a bolted-on drag would get wrong.
    let (mut runtime, source, sink) = rendered();
    runtime.register_drop_target(DropTarget::new(sink));

    // A press first, so a press anchor exists to be activated by a release.
    runtime.handle_pointer(at(PointerEventKind::Down, OVER_SINK));
    runtime.begin_drag(text_drag(source));
    let dispatch = runtime.handle_pointer(at(PointerEventKind::Up, OVER_SINK));

    assert!(
        dispatch.command.is_none(),
        "the release dropped; it must not also activate"
    );
    assert!(matches!(
        runtime.take_drag_outcome().expect("it finished"),
        DragOutcome::Dropped { .. }
    ));
}

#[test]
fn ordinary_pointer_input_is_untouched_when_no_drag_is_in_flight() {
    // The regression guard that matters: hovering and clicking are the common
    // case and must behave exactly as before.
    let (mut runtime, _source, sink) = rendered();
    runtime.register_drop_target(DropTarget::new(sink));

    let dispatch = runtime.handle_pointer(at(PointerEventKind::Move, OVER_SINK));
    assert_eq!(dispatch.target, Some(sink), "hit-testing still works");
    assert!(dispatch.hover_changed, "and hover still tracks");
    assert!(runtime.drag_session().is_none());
    assert!(runtime.take_drag_outcome().is_none());
}
