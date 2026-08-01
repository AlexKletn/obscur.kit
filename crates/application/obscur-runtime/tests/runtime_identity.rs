#![doc = include_str!("runtime_identity.doc/_module.md")]
#![forbid(unsafe_code)]

use obscur_geometry::{Point, Size};
use obscur_testkit::fixture::{CounterPanel, TestFixture};
use obscur_testkit::selector::NodePath;

#[test]
fn new_starts_idle_with_identity_and_viewport() {
    let runtime = TestFixture::runtime();
    assert_eq!(runtime.runtime_id(), TestFixture::RUNTIME_ID);
    assert_eq!(runtime.id(), TestFixture::RUNTIME_ID);
    assert_eq!(runtime.viewport(), TestFixture::VIEWPORT);
    assert!(!runtime.has_pending_frame());
    assert_eq!(runtime.pending_frames(), 0);
    assert!(runtime.layout_boxes().is_empty());
}

#[test]
fn layout_boxes_are_retained_after_a_frame() {
    let mut runtime = TestFixture::runtime();
    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();

    let _ = runtime.render_now(&view, std::slice::from_ref(&sheet));

    let child = NodePath::new(&[0]).resolve(&runtime);
    let boxes = runtime.layout_boxes();
    assert!(!boxes.is_empty(), "boxes retained from the frame");
    let child_box = boxes.get(&child).expect("the sized node has a box");
    // `.counter` is 100x50 per the stylesheet, arranged at the origin.
    assert_eq!(child_box.size(), Size::new(100.0, 50.0));
    assert_eq!(child_box.origin(), Point::new(0.0, 0.0));
}

#[test]
fn render_now_is_deterministic() {
    let mut a = TestFixture::runtime();
    let mut b = TestFixture::runtime();
    let view = CounterPanel::view();
    let sheet = CounterPanel::sheet();

    let first = a.render_now(&view, std::slice::from_ref(&sheet));
    let second = b.render_now(&view, std::slice::from_ref(&sheet));
    let first_commands: Vec<_> = first.snapshot().display_list().commands().to_vec();
    let second_commands: Vec<_> = second.snapshot().display_list().commands().to_vec();
    assert_eq!(first_commands, second_commands);
}

#[test]
fn set_viewport_changes_layout_bounds() {
    let mut runtime = TestFixture::runtime();
    assert_eq!(runtime.viewport(), TestFixture::VIEWPORT);
    runtime.set_viewport(Size::new(640.0, 480.0));
    assert_eq!(runtime.viewport(), Size::new(640.0, 480.0));
}

#[test]
fn debug_reports_identity_fields() {
    let runtime = TestFixture::runtime();
    let rendered = format!("{runtime:?}");
    assert!(rendered.contains("UiRuntime"));
    assert!(rendered.contains("next_frame"));
    assert!(rendered.contains("pending_frames"));
}
