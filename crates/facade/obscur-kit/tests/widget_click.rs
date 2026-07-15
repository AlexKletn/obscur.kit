//! End-to-end proof that the **widget library is clickable**: a real
//! [`Button`](obscur_kit::widgets::button::Button), rendered through the
//! embeddable [`UiRuntime`](obscur_kit::runtime::UiRuntime), dispatches its bound
//! command when the runtime detects a pointer click on it — and the reactive
//! re-render loop rebuilds the view from the mutated state.
//!
//! This exercises the fix that aligns the `Button`'s click binding to the
//! reserved [`EVENT_CLICK`](obscur_kit::view::view::EVENT_CLICK) event symbol the
//! runtime's click resolver looks up. Before the fix the button bound a
//! widget-local event symbol, so a runtime click on it resolved to no command.
#![forbid(unsafe_code)]

use std::cell::Cell;
use std::rc::Rc;

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{InternedSymbol, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::frame::FrameReason;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewProperty, ViewValue};
use obscur_kit::widgets::button::Button;

/// App-owned increment command the counter button binds and the app registers.
const CMD_INC: InternedSymbol = InternedSymbol::from_raw(0x1_C0DE);

/// Builds the counter view from `count`: a **real** [`Button`] whose label is the
/// current count and whose click fires `CMD_INC`. The button's own node is tagged
/// with the `btn` class so the stylesheet can size it into a hit-testable box.
fn counter_view(count: i32) -> View {
    let node = Button::new(count.to_string(), CMD_INC)
        .view()
        .into_fragment()
        .nodes()
        .to_vec()
        .remove(0)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("btn".into())));
    View::single(node)
}

/// `.btn { width: 100px; height: 50px; background-color: green }` — sizes the
/// button element to a 100x50 box at the origin so a pointer at its centre
/// (`(50, 25)`) hits it.
fn button_sheet() -> Stylesheet {
    let css = ".btn { width: 100px; height: 50px; background-color: #5cb85c; }";
    let parsed = StylesheetParser::new(&CssSource::anonymous(css)).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// A left-button pointer event of `kind` at `(x, y)`.
fn mouse(kind: PointerEventKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        kind,
        Point::new(x, y),
    )
}

/// The content of the first text node in the retained tree — the button's label,
/// which is the rendered counter value the reactive loop rebuilt.
fn tree_text(runtime: &UiRuntime) -> String {
    runtime
        .tree()
        .iter()
        .find_map(|(_, node)| node.kind().text().map(str::to_owned))
        .expect("the retained tree has a text node after a frame")
}

#[test]
fn clicking_a_real_button_through_the_runtime_dispatches_and_re_renders() {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(300.0, 300.0));

    // Shared counter state: the click handler mutates it, the view builder reads
    // it — the canonical `Rc<Cell<i32>>` bridge.
    let count = Rc::new(Cell::new(0));
    {
        let c = count.clone();
        runtime.on_command(CMD_INC, move || c.set(c.get() + 1));
    }
    {
        let c = count.clone();
        runtime.set_root_view(move || counter_view(c.get()));
    }
    runtime.set_stylesheets(vec![button_sheet()]);

    // First reactive frame: the button's label reads "0".
    runtime.request_frame(FrameReason::UiCommit);
    runtime
        .run_due_frame_reactive()
        .expect("the first reactive frame is due");
    assert_eq!(tree_text(&runtime), "0", "the counter starts at 0");

    // Click the button (Down then Up on the same node at its centre). This is the
    // WIDGET-level proof: the runtime resolves the click to the button's bound
    // command via the reserved `EVENT_CLICK` symbol and dispatches it.
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    let up = runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
    assert_eq!(
        up.command(),
        Some(CMD_INC),
        "clicking the Button resolved to its bound command"
    );
    assert_eq!(count.get(), 1, "the registered handler mutated the state");
    assert!(
        runtime.has_pending_frame(),
        "the dispatch requested a frame"
    );

    // Reactive re-render: the builder rebuilds the button from the mutated state,
    // so its label — the retained tree's text — now shows "1".
    runtime
        .run_due_frame_reactive()
        .expect("the click-requested reactive frame is due");
    assert_eq!(tree_text(&runtime), "1", "the rebuilt button shows 1");

    // Repeatable: a second click drives the loop again to "2".
    runtime.handle_pointer(mouse(PointerEventKind::Down, 50.0, 25.0));
    runtime.handle_pointer(mouse(PointerEventKind::Up, 50.0, 25.0));
    assert_eq!(count.get(), 2);
    runtime
        .run_due_frame_reactive()
        .expect("the second click's reactive frame is due");
    assert_eq!(tree_text(&runtime), "2", "the click loop is repeatable");
}
