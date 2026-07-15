//! End-to-end: the in-window inspect overlay (spec §7.6) highlights the hovered
//! node's box and the runtime reports its readout, driven through the real
//! render + hit-test path (render → `handle_pointer` → hover → compose overlay).
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::{Point, Size};
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::interaction::pointer::{PointerDevice, PointerEvent, PointerEventKind, PointerId};
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// A `#root` container holding one `.box` child.
fn view() -> View {
    let root = ComponentId::from_raw(0x00C0_0001);
    let box_child =
        ViewNode::element(root).with_prop(ViewProperty::new(CLASS, ViewValue::Text("box".into())));
    let root_node = ViewNode::element(root)
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("root".into())))
        .with_children(ViewFragment::from_nodes(vec![box_child]));
    View::from_fragment(ViewFragment::from_nodes(vec![root_node]))
}

fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

/// Moves the pointer to `(x, y)` so the runtime hit-tests and updates hover.
fn hover(runtime: &mut UiRuntime, x: f32, y: f32) {
    runtime.handle_pointer(PointerEvent::new(
        PointerId::MOUSE,
        PointerDevice::Mouse,
        PointerEventKind::Move,
        Point::new(x, y),
    ));
}

fn draw_path_count(list: &obscur_kit::scene::command::DisplayList) -> usize {
    list.commands()
        .iter()
        .filter(|c| matches!(c, PaintCommand::DrawPath { .. }))
        .count()
}

#[test]
fn inspect_overlay_adds_a_highlight_over_the_hovered_box() {
    let sheet =
        compile("#root { width: 200px; height: 200px } .box { width: 100px; height: 60px }");
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));

    // First frame lays the boxes out so the hit-test has geometry.
    runtime.render_now(&view(), std::slice::from_ref(&sheet));
    // Hover inside the `.box` (at 0,0 → 100,60).
    hover(&mut runtime, 40.0, 30.0);

    // Baseline: inspect off — a known number of DrawPath commands.
    let baseline = runtime.render_now(&view(), std::slice::from_ref(&sheet));
    let baseline_paths = draw_path_count(baseline.snapshot().display_list());

    // Turn the inspector on and re-render: the overlay adds exactly two
    // DrawPath commands (a translucent fill under a solid outline).
    runtime.set_inspect_mode(true);
    assert!(runtime.inspect_mode());
    let inspected = runtime.render_now(&view(), std::slice::from_ref(&sheet));
    let inspected_paths = draw_path_count(inspected.snapshot().display_list());

    assert_eq!(
        inspected_paths,
        baseline_paths + 2,
        "inspect overlay should add a fill + outline over the hovered box"
    );
    assert!(inspected.snapshot().display_list().is_balanced());

    // Turning it back off returns to the baseline command count.
    runtime.set_inspect_mode(false);
    let off = runtime.render_now(&view(), std::slice::from_ref(&sheet));
    assert_eq!(
        draw_path_count(off.snapshot().display_list()),
        baseline_paths
    );
}

#[test]
fn inspect_snapshot_dumps_the_tree_with_geometry_and_styles() {
    // The structured observe half of the agent loop: one call yields the whole
    // tree as data an agent can assert on, without hovering or a window.
    let sheet = compile(
        "#root { width: 200px; height: 200px } \
         .box { width: 100px; height: 60px; background-color: #22d3a6 }",
    );
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    runtime.render_now(&view(), std::slice::from_ref(&sheet));

    let snapshot = runtime.inspect_snapshot();
    assert!(snapshot.len() >= 2, "root + box at least");

    // The `.box` node is present with its resolved absolute geometry and styles.
    let box_node = snapshot
        .nodes
        .iter()
        .find(|node| node.classes == vec!["box".to_owned()])
        .expect("the .box node is in the dump");
    let bounds = box_node.bounds.expect("the box was laid out");
    assert!((bounds.size.width - 100.0).abs() < 1e-3, "{bounds:?}");
    assert!((bounds.size.height - 60.0).abs() < 1e-3, "{bounds:?}");
    let style_names: Vec<&str> = box_node
        .styles
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    assert!(
        style_names.contains(&"Width") && style_names.contains(&"BackgroundColor"),
        "expected width + background in the dumped styles, got {:?}",
        box_node.styles
    );

    // The Display renders an indented tree an agent (or a log) can read directly.
    let rendered = snapshot.to_string();
    assert!(rendered.contains(".box"), "{rendered}");
    assert!(rendered.contains("100×60"), "{rendered}");
}

#[test]
fn hovered_inspect_reports_the_box_identity_and_geometry() {
    let sheet =
        compile("#root { width: 200px; height: 200px } .box { width: 100px; height: 60px }");
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(200.0, 200.0));
    runtime.render_now(&view(), std::slice::from_ref(&sheet));

    // Nothing hovered yet.
    assert!(runtime.hovered_inspect().is_none());

    hover(&mut runtime, 40.0, 30.0);
    let info = runtime.hovered_inspect().expect("a hovered node");
    assert_eq!(info.classes, vec!["box".to_owned()]);
    assert_eq!(info.component, Some(ComponentId::from_raw(0x00C0_0001)));
    // The `.box` is 100×60 at the flow origin.
    assert!(
        (info.bounds.size.width - 100.0).abs() < 1e-3,
        "{:?}",
        info.bounds
    );
    assert!(
        (info.bounds.size.height - 60.0).abs() < 1e-3,
        "{:?}",
        info.bounds
    );

    // The readout line names the class and geometry.
    let line = info.to_string();
    assert!(line.contains(".box"), "{line}");
    assert!(line.contains("100×60"), "{line}");
}
