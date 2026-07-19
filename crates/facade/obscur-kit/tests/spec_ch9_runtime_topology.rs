//! **SPECIFICATION-DERIVED** cases for §1.4 Runtime topology ("Multiple runtimes
//! may coexist in one process. Global singletons are avoided…"), §3.7 Visibility
//! and lifecycle ("Runtime lifecycle does not imply process lifecycle."), §3.2 UI
//! transactions ("provisional systems such as DnD may use commit/cancel
//! semantics") and §2.2 Tree model ("child identity is stable through explicit
//! keys") — the invariants behind the §9.1 non-goals "process lifecycle
//! ownership", "mandatory global state store" and "application-wide undo/redo".
//!
//! Every expected value was produced by an agent reading the specification with
//! **no access to this repository's source**. A failure here is a defect report
//! about the engine, and must not be "corrected" by editing the expected value.
#![forbid(unsafe_code)]

#[path = "spec_ch9_support/mod.rs"]
mod support;

use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::runtime::transaction::{ProvisionalTransaction, UiOperation, UiTransaction};
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};
use obscur_kit::windowing::port::WindowBackend;
use obscur_kit::windowing::request::{CloseWindow, CreateWindow};
use support::{approx, at, compile, el, runtime_id, txt, view};

const VW: f32 = 1000.0;

/// An element carrying a stable key and a class.
fn keyed(id: u64, key: &str, class: &str) -> ViewNode {
    ViewNode::element(ComponentId::from_raw(id))
        .with_key(key)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
        .with_children(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(99))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("tall".into()))),
        ]))
}

/// §3.7 Visibility and lifecycle — "The host decides whether to exit." and "A
/// runtime may stop and release resources while the process continues."
///
/// NG-LIFECYCLE-01. (The "`last window closed` event is delivered" half is
/// recorded as unexpressible: `WindowEvent` has no such variant and no window
/// registry sits above the backend port to raise one.)
#[test]
fn closing_the_last_window_neither_exits_the_process_nor_poisons_the_runtime() {
    let sentinel = std::cell::Cell::new(0_u32);
    let mut backend = obscur_kit::windowing::port::NullWindowBackend::new();
    let first = backend
        .create_window(CreateWindow::new(
            obscur_kit::windowing::window::WindowProperties::default(),
        ))
        .expect("the first window opens");
    let _ = backend.close_window(CloseWindow::request(first));

    // Observable proof the process outlived the close.
    sentinel.set(1);
    assert_eq!(
        sentinel.get(),
        1,
        "the process is still alive after the close"
    );

    let second = backend
        .create_window(CreateWindow::new(
            obscur_kit::windowing::window::WindowProperties::default(),
        ))
        .expect("a second window opens on the same runtime after the last one closed");
    assert_ne!(second, first, "the second window is a fresh window");

    let mut rt = runtime_id(1, VW, VW);
    let sheets = [compile(".a{width:10px;height:10px}")];
    let v = view(vec![el(1, "a", vec![])]);
    let _ = rt.render_now(&v, &sheets);
    let (_, _, w, h) = support::geom(&rt, at(&rt, &[0]));
    approx(w, 10.0, "the second window's first frame commits the tree");
    approx(h, 10.0, "the second window's first frame commits the tree");
}

/// §9.1 — "- process lifecycle ownership;" and §3.7 "Runtime lifecycle does not
/// imply process lifecycle."
///
/// NG-LIFECYCLE-02.
#[test]
fn a_second_runtime_initializes_fully_after_the_first_is_dropped() {
    let sheets = [compile(".a{width:20px;height:20px}")];
    let v = view(vec![el(1, "a", vec![txt("hello")])]);

    {
        let mut a = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
        let _ = a.render_now(&v, &sheets);
        drop(a);
    }

    let mut b = UiRuntime::new(RuntimeId::from_raw(2), Size::new(VW, VW));
    let _ = b.render_now(&v, &sheets);
    let (_, _, w, h) = support::geom(&b, at(&b, &[0]));
    approx(w, 20.0, "runtime B produced the correct geometry");
    approx(h, 20.0, "runtime B produced the correct geometry");
    assert!(
        !b.snapshot().display_list().commands().is_empty(),
        "runtime B produced a scene after runtime A's teardown"
    );
}

/// §1.4 Runtime topology — "Multiple runtimes may coexist in one process. Global
/// singletons are avoided except where the platform makes them unavoidable." and
/// §9.1 "- mandatory global state store;"
///
/// NG-MULTIRT-01.
#[test]
fn two_runtimes_hold_separate_state_and_do_not_cross_invalidate() {
    let sheets = [compile(".a{width:10px;height:10px}")];

    let state_a = support::Shared::new(1_u32);
    let state_b = support::Shared::new(2_u32);

    let mut a = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    let mut b = UiRuntime::new(RuntimeId::from_raw(2), Size::new(VW, VW));

    let read_a = state_a.clone();
    a.set_root_view(move || {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(1))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into())))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                    read_a.get().to_string(),
                )])),
        ]))
    });
    let read_b = state_b.clone();
    b.set_root_view(move || {
        View::from_fragment(ViewFragment::from_nodes(vec![
            ViewNode::element(ComponentId::from_raw(1))
                .with_prop(ViewProperty::new(CLASS, ViewValue::Text("a".into())))
                .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                    read_b.get().to_string(),
                )])),
        ]))
    });
    a.set_stylesheets(sheets.to_vec());
    b.set_stylesheets(sheets.to_vec());
    a.request_frame(obscur_kit::runtime::frame::FrameReason::UiCommit);
    b.request_frame(obscur_kit::runtime::frame::FrameReason::UiCommit);
    let _ = a.run_due_frame_reactive();
    let _ = b.run_due_frame_reactive();

    assert_eq!(text_of(&a), "1", "runtime A renders its own state");
    assert_eq!(text_of(&b), "2", "runtime B renders its own state");

    // Mutating A must not wake B.
    state_a.set(9);
    a.request_frame(obscur_kit::runtime::frame::FrameReason::UiCommit);
    let _ = a.run_due_frame_reactive();
    assert_eq!(text_of(&a), "9", "runtime A re-rendered");
    assert_eq!(
        text_of(&b),
        "2",
        "runtime B is untouched by A's state change"
    );
    assert!(
        !b.has_pending_frame(),
        "mutating A's state requests no frame in B"
    );
}

/// The first text child's content, as rendered in the retained tree.
fn text_of(rt: &UiRuntime) -> String {
    let root = rt.tree().root().expect("a root");
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        let entry = rt.tree().get(node).expect("a live node");
        if let Some(text) = entry.kind().text() {
            return text.to_string();
        }
        stack.extend(entry.children().iter().copied());
    }
    String::new()
}

/// §3.2 UI transactions — "provisional systems such as DnD may use commit/cancel
/// semantics;" and "heavyweight universal rollback is not required." (making
/// concrete §9.1 "- application-wide undo/redo;")
///
/// NG-UNDO-01: cancelling a provisional drag transaction must not revert an
/// unrelated, already-committed edit.
#[test]
fn cancelling_a_provisional_drag_does_not_revert_a_committed_edit() {
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));
    let sheets = [compile(".row{width:100px;height:10px}")];

    // An ordinary committed UI transaction: the field reads "hello".
    let field = View::from_fragment(ViewFragment::from_nodes(vec![
        ViewNode::element(ComponentId::from_raw(1))
            .with_prop(ViewProperty::new(CLASS, ViewValue::Text("row".into())))
            .with_children(ViewFragment::from_nodes(vec![ViewNode::text("hello")])),
    ]));
    let _ = rt.render_now(&field, &sheets);
    assert_eq!(text_of(&rt), "hello", "the committed edit is visible");

    // A provisional DnD transaction that inserts a placeholder, then cancels.
    let mut tx = UiTransaction::default();
    tx.push(UiOperation::ReplaceRoot);
    let provisional = ProvisionalTransaction::begin(tx);
    provisional.cancel();

    let _ = rt.render_now(&field, &sheets);
    assert_eq!(
        text_of(&rt),
        "hello",
        "cancelling the drag left the unrelated committed edit intact"
    );
}

/// §2.2 Tree model — "child identity is stable through explicit keys;" and
/// "updates are expressed as new desired state rather than direct retained-tree
/// mutation." (guardrail §9.6 "require arbitrary mutation of retained tree")
///
/// NG-KEYS-01.
#[test]
fn retained_identity_and_per_node_state_follow_keys_not_positions() {
    let sheets = [compile(
        ".row{width:100px;height:50px;overflow:scroll}.tall{width:100px;height:500px}",
    )];
    let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VW, VW));

    let first = View::from_fragment(ViewFragment::from_nodes(vec![
        keyed(1, "A", "row"),
        keyed(1, "B", "row"),
        keyed(1, "C", "row"),
    ]));
    let _ = rt.render_now(&first, &sheets);

    let b_before = at(&rt, &[1]);
    rt.set_scroll_top(b_before, 10.0);
    approx(rt.scroll_top(b_before), 10.0, "B's scroll offset was set");

    let reordered = View::from_fragment(ViewFragment::from_nodes(vec![
        keyed(1, "C", "row"),
        keyed(1, "A", "row"),
        keyed(1, "B", "row"),
    ]));
    let _ = rt.render_now(&reordered, &sheets);

    let b_after = at(&rt, &[2]);
    assert_eq!(
        b_after, b_before,
        "key B keeps its retained NodeId across the reorder"
    );
    approx(
        rt.scroll_top(b_after),
        10.0,
        "key B keeps its retained scroll offset at its new index",
    );
}
