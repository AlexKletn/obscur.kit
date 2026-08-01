#![doc = include_str!("reactive_rendering.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_identity::{ComponentId, InternedSymbol};
use obscur_interaction::pointer::PointerEventKind;
use obscur_runtime::frame::FrameReason;
use obscur_testkit::fixture::{InteractionPanel, TestFixture, mouse};
use obscur_testkit::selector::SemanticSelector;
use obscur_view::view::{
    CLASS, EVENT_CLICK, View, ViewEventBinding, ViewFragment, ViewNode, ViewProperty, ViewValue,
};

/// App-owned increment command the reactive counter binds and registers.
const CMD_INC: InternedSymbol = InternedSymbol::from_raw(0x1C0);

/// A reactive counter view: a `panel` (100x100) holding a single `b`
/// (100x50, at y∈[0,50)) that binds `on-click → CMD_INC` and contains a text
/// node showing `count`. A click anywhere in `b` (e.g. `(50, 25)`) fires the
/// increment command; the text node is what the assertions read back.
fn counter_reactive_view(count: i32) -> View {
    View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(300))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("panel".into())))
            .with_children(ViewFragment::from_nodes(vec![
                ViewNode::element(ComponentId::from_raw(301))
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("b".into())))
                    .with_event(ViewEventBinding::new(EVENT_CLICK, CMD_INC))
                    .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                        count.to_string(),
                    )])),
            ])),
    ]))
}

#[test]
fn reactive_loop_rebuilds_view_on_click_end_to_end() {
    let mut runtime = TestFixture::runtime();

    // The counter state shared between the click handler (which mutates it)
    // and the view builder (which reads it) — an `Rc<Cell<i32>>`.
    let count = Rc::new(Cell::new(0));
    {
        let c = count.clone();
        runtime.on_command(CMD_INC, move || c.set(c.get() + 1));
    }
    {
        let c = count.clone();
        runtime.set_root_view(move || counter_reactive_view(c.get()));
    }
    runtime.set_stylesheets(vec![InteractionPanel::sheet()]);

    // First reactive frame: schedule one, run it, the tree shows "0".
    runtime.request_frame(FrameReason::UiCommit);
    let first = runtime
        .run_due_frame_reactive()
        .expect("the first reactive frame is due");
    assert!(first.reconciled(), "the first reactive frame reconciles");
    assert_eq!(
        SemanticSelector::first_text(&runtime),
        "0",
        "the counter starts at 0"
    );

    // Click on `b` (50, 25): Down then Up on the same node dispatches the
    // increment command, which bumps the state and requests a frame.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
    assert_eq!(up.command(), Some(CMD_INC), "the click fired the increment");
    assert_eq!(count.get(), 1, "the handler mutated the shared state");
    assert!(runtime.has_pending_frame(), "the click requested a frame");

    // Second reactive frame: the builder REBUILDS from the mutated state, so
    // the reconciled tree now shows "1" — proving click→state→re-render.
    runtime
        .run_due_frame_reactive()
        .expect("the click-requested reactive frame is due");
    assert_eq!(
        SemanticSelector::first_text(&runtime),
        "1",
        "the rebuilt view shows 1"
    );

    // A second click → the loop runs again → the tree shows "2".
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
    assert_eq!(count.get(), 2);
    runtime
        .run_due_frame_reactive()
        .expect("the second click's reactive frame is due");
    assert_eq!(
        SemanticSelector::first_text(&runtime),
        "2",
        "the loop is repeatable"
    );
}

#[test]
fn reactive_frame_re_renders_a_changed_builder_output() {
    let mut runtime = TestFixture::runtime();
    // Drive the builder's output directly (no click) to prove the reactive
    // frame reflects whatever the builder returns each time it is called.
    let value = Rc::new(Cell::new(7));
    {
        let v = value.clone();
        runtime.set_root_view(move || counter_reactive_view(v.get()));
    }
    runtime.set_stylesheets(vec![InteractionPanel::sheet()]);

    runtime.request_frame(FrameReason::UiCommit);
    runtime.run_due_frame_reactive().expect("first frame due");
    assert_eq!(SemanticSelector::first_text(&runtime), "7");

    // Mutate the state the builder reads, request another frame, run it: the
    // rebuilt tree reflects the new value.
    value.set(42);
    runtime.request_frame(FrameReason::UiCommit);
    runtime.run_due_frame_reactive().expect("second frame due");
    assert_eq!(
        SemanticSelector::first_text(&runtime),
        "42",
        "the builder's new output rendered"
    );
}

#[test]
fn reactive_frame_with_no_builder_returns_none() {
    let mut runtime = TestFixture::runtime();
    assert!(!runtime.has_root_view());
    // A frame is pending, but with no builder the reactive path is a no-op
    // that returns None and leaves the pending request untouched.
    runtime.request_frame(FrameReason::UiCommit);
    assert!(runtime.run_due_frame_reactive().is_none());
    assert!(
        runtime.has_pending_frame(),
        "the no-op reactive path did not drain the pending frame"
    );
}

#[test]
fn reactive_frame_returns_none_when_nothing_is_due() {
    let mut runtime = TestFixture::runtime();
    runtime.set_root_view(|| counter_reactive_view(0));
    runtime.set_stylesheets(vec![InteractionPanel::sheet()]);
    // No frame requested → nothing due → None, no rebuild, no side effects.
    assert!(!runtime.has_pending_frame());
    assert!(runtime.run_due_frame_reactive().is_none());
    assert!(runtime.tree().root().is_none(), "no frame ran");
}

#[test]
fn explicit_view_path_is_unaffected_by_an_installed_builder() {
    let mut runtime = TestFixture::runtime();
    // Install a builder that would render "99", then drive the EXPLICIT
    // `&View` path with a different view: the explicit path must ignore the
    // held builder entirely and render what it was handed.
    runtime.set_root_view(|| counter_reactive_view(99));
    let sheet = InteractionPanel::sheet();
    let sheets = std::slice::from_ref(&sheet);

    let result = runtime.render_now(&counter_reactive_view(3), sheets);
    assert!(result.reconciled());
    assert_eq!(
        SemanticSelector::first_text(&runtime),
        "3",
        "render_now rendered the passed view, not the builder's"
    );

    // The scheduler-gated `&View` path is likewise unaffected.
    runtime.request_frame(FrameReason::UiCommit);
    runtime
        .run_due_frame(&counter_reactive_view(5), sheets)
        .expect("frame due");
    assert_eq!(
        SemanticSelector::first_text(&runtime),
        "5",
        "run_due_frame used the passed view"
    );
}
