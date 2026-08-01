#![doc = include_str!("frame_phases.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_identity::ComponentId;
use obscur_runtime::frame::FrameReason;
use obscur_style::selector::PseudoState;
use obscur_testkit::fixture::{HoverBox, TestFixture};
use obscur_testkit::selector::NodePath;
use obscur_view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A hover that can only repaint must not relayout.
#[test]
fn a_paint_only_hover_reuses_the_retained_boxes() {
    // THE incremental-invalidation win (§3.4 p5, §4.3): the plan says an
    // `Input` frame may relayout (it is classified from the REASON alone),
    // but the hover rule only changes `background-color` — a paint-only
    // property — so no box can have moved and the whole layout phase is
    // skipped, reusing the retained boxes.
    let view = HoverBox::view();
    let sheet = HoverBox::paint_hover_sheet();
    let sheets = std::slice::from_ref(&sheet);
    let mut runtime = TestFixture::runtime();
    runtime.render_now(&view, sheets);
    let node = NodePath::new(&[0]).resolve(&runtime);
    let cold_box = runtime
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("a laid-out box");

    runtime.set_pseudo_state(node, PseudoState::HOVER);
    runtime.request_frame(FrameReason::Input);
    let hot = runtime
        .run_due_frame(&view, sheets)
        .expect("the input frame is due");

    assert!(
        !hot.relaid_out(),
        "a paint-only hover must not relayout — the retained boxes are provably valid"
    );
    assert!(
        hot.restyled(),
        "it still restyles so the fill reaches paint"
    );
    assert!(hot.composed(), "and still re-composes");
    assert_eq!(
        runtime.layout_boxes().get(&node).copied(),
        Some(cold_box),
        "the retained box carried over unchanged"
    );
}

/// The other side of the classification: a hover that moves a box does relayout.
#[test]
fn a_layout_affecting_hover_still_relayouts() {
    // The other side of the classification: `.box:hover { width: 200px }`
    // changes a layout-affecting property, so the layout phase must run.
    let view = HoverBox::view();
    let sheet = HoverBox::layout_hover_sheet();
    let sheets = std::slice::from_ref(&sheet);
    let mut runtime = TestFixture::runtime();
    runtime.render_now(&view, sheets);
    let node = NodePath::new(&[0]).resolve(&runtime);

    runtime.set_pseudo_state(node, PseudoState::HOVER);
    runtime.request_frame(FrameReason::Input);
    let hot = runtime
        .run_due_frame(&view, sheets)
        .expect("the input frame is due");

    assert!(hot.relaid_out(), "a width change must relayout");
    assert_eq!(
        runtime
            .layout_boxes()
            .get(&node)
            .map(|b| b.bounds.size.width),
        Some(200.0),
        "the hovered box grew to its hover width"
    );
}

/// The same win has to hold on the path a windowed host actually drives.
#[test]
fn a_paint_only_hover_is_layout_free_on_the_reactive_path_too() {
    // The refinement has to bite where hosts actually live. A windowed host
    // drives every frame through `run_due_frame_reactive`, which classifies
    // ALL of them as a structural `UiCommit` — so gating on the plan's
    // `reconcile()` flag would relayout on every hover and the whole
    // optimization would be dead code in real windows. The gate is the
    // reconcile's ACTUAL result: an unchanged rebuild commits empty.
    let sheet = HoverBox::paint_hover_sheet();
    let mut runtime = TestFixture::runtime();
    runtime.set_root_view(HoverBox::view);
    runtime.set_stylesheets(vec![sheet]);
    runtime.request_frame(FrameReason::UiCommit);
    let first = runtime
        .run_due_frame_reactive()
        .expect("the first frame is due");
    assert!(first.relaid_out(), "the first frame arranges the boxes");

    let node = NodePath::new(&[0]).resolve(&runtime);
    let cold_box = runtime
        .layout_boxes()
        .get(&node)
        .copied()
        .expect("a laid-out box");

    runtime.set_pseudo_state(node, PseudoState::HOVER);
    runtime.request_frame(FrameReason::UiCommit);
    let hot = runtime
        .run_due_frame_reactive()
        .expect("the hover frame is due");

    assert!(
        !hot.reconciled(),
        "the rebuilt view is identical: reconcile commits empty"
    );
    assert!(
        !hot.relaid_out(),
        "a paint-only hover must be layout-free on the reactive path as well"
    );
    assert!(
        hot.restyled(),
        "it still restyles so the fill reaches paint"
    );
    assert_eq!(
        runtime.layout_boxes().get(&node).copied(),
        Some(cold_box),
        "the retained box carried over unchanged"
    );
}

/// The safety side of the same gate: changed text is a real change.
#[test]
fn a_reactive_rebuild_that_changes_text_still_relayouts() {
    // The safety side of the same gate: reconcile diffs text content BY VALUE,
    // so a builder whose state produced new text commits a real change and the
    // frame must lay out — text is not a computed style, and skipping here
    // would strand the old text box (the wireframe's status line does exactly
    // this on every click).
    let label = Rc::new(RefCell::new("one".to_owned()));
    let sheet = HoverBox::paint_hover_sheet();
    let mut runtime = TestFixture::runtime();
    {
        let label = label.clone();
        runtime.set_root_view(move || {
            View::from_fragment(ViewFragment::from_nodes(vec![
                ViewNode::element(ComponentId::from_raw(10))
                    .with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())))
                    .with_children(ViewFragment::from_nodes(vec![ViewNode::text(
                        label.borrow().as_str(),
                    )])),
            ]))
        });
    }
    runtime.set_stylesheets(vec![sheet]);
    runtime.request_frame(FrameReason::UiCommit);
    runtime
        .run_due_frame_reactive()
        .expect("the first frame is due");

    // An identical rebuild: provably layout-free (the control for the assert
    // below — it proves the next frame's relayout is caused by the TEXT).
    runtime.request_frame(FrameReason::UiCommit);
    let same = runtime
        .run_due_frame_reactive()
        .expect("the repeat frame is due");
    assert!(!same.relaid_out(), "an identical rebuild is layout-free");

    // Now the state changes the text the builder emits.
    *label.borrow_mut() = "two".to_owned();
    runtime.request_frame(FrameReason::UiCommit);
    let changed = runtime
        .run_due_frame_reactive()
        .expect("the commit frame is due");
    assert!(changed.reconciled(), "the new text is a real tree change");
    assert!(changed.relaid_out(), "changed text must relayout");
}
